use crate::internal_prelude::*;

pub(crate) type ParseResult<T> = core::result::Result<T, ParseError>;

#[allow(unused)]
pub(crate) trait ParseResultExt<T> {
    /// This is not a `From` because it wants to be explicit
    fn convert_to_final_result(self) -> syn::Result<T>;
    fn add_context_if_error_and_no_context(self, context: impl FnOnce() -> String) -> Self;
    fn into_execution_result(self) -> ExecutionResult<T>;
}

impl<T> ParseResultExt<T> for ParseResult<T> {
    fn convert_to_final_result(self) -> syn::Result<T> {
        self.map_err(|error| error.convert_to_final_error())
    }

    fn add_context_if_error_and_no_context(self, context: impl FnOnce() -> String) -> Self {
        self.map_err(|error| error.add_context_if_none(context()))
    }

    fn into_execution_result(self) -> ExecutionResult<T> {
        self.map_err(|error| error.into())
    }
}

#[derive(Debug)]
pub(crate) enum ParseError {
    Standard(syn::Error),
    Contextual(syn::Error, String),
}

impl From<syn::Error> for ParseError {
    fn from(e: syn::Error) -> Self {
        ParseError::Standard(e)
    }
}

impl HasSpan for ParseError {
    fn span(&self) -> Span {
        match self {
            ParseError::Standard(e) => e.span(),
            ParseError::Contextual(e, _) => e.span(),
        }
    }
}

impl ParseError {
    /// This is not a `From` because it wants to be explicit
    pub(crate) fn convert_to_final_error(self) -> syn::Error {
        match self {
            ParseError::Standard(e) => e,
            ParseError::Contextual(e, message) => e.concat(&format!("\n{}", message)),
        }
    }

    pub(crate) fn add_context_if_none(self, context: impl std::fmt::Display) -> Self {
        match self {
            ParseError::Standard(e) => ParseError::Contextual(e, context.to_string()),
            other => other,
        }
    }

    pub(crate) fn context(&self) -> Option<&str> {
        match self {
            ParseError::Standard(_) => None,
            ParseError::Contextual(_, context) => Some(context),
        }
    }
}

// Ideally this would be our own enum with Completed / Interrupted variants,
// but we want it to work with `?` and defining custom FromResidual is not
// possible on stable (at least according to our MSRV).
pub(crate) type ExecutionResult<T> = core::result::Result<T, ExecutionInterrupt>;

pub(crate) enum ExecutionOutcome<T> {
    Value(T),
    ControlFlow(ControlFlowInterrupt),
}

pub(crate) trait ExecutionResultExt<T> {
    fn catch_control_flow(
        self,
        interpreter: &mut Interpreter,
        should_catch: impl FnOnce(&ControlFlowInterrupt) -> bool,
        catch_at_scope: ScopeId,
    ) -> ExecutionResult<ExecutionOutcome<T>>;

    fn catch_execution_error_at_same_scope(self) -> ExecutionResult<Result<T, syn::Error>>;

    /// This is not a `From` because it wants to be explicit
    fn convert_to_final_result(self) -> syn::Result<T>;
}

impl<T> ExecutionResultExt<T> for ExecutionResult<T> {
    fn catch_control_flow(
        self,
        interpreter: &mut Interpreter,
        should_catch: impl FnOnce(&ControlFlowInterrupt) -> bool,
        return_to_scope: ScopeId,
    ) -> ExecutionResult<ExecutionOutcome<T>> {
        interpreter.catch_control_flow(self, should_catch, return_to_scope)
    }

    fn catch_execution_error_at_same_scope(self) -> ExecutionResult<Result<T, syn::Error>> {
        match self {
            Ok(value) => Ok(Ok(value)),
            Err(interrupt) => interrupt.into_execution_error::<T>(),
        }
    }

    fn convert_to_final_result(self) -> syn::Result<T> {
        self.map_err(|error| error.convert_to_final_error())
    }
}

#[derive(Debug)]
pub(crate) struct ExecutionInterrupt {
    inner: Box<ExecutionInterruptInner>,
}

impl ExecutionInterrupt {
    fn new(inner: ExecutionInterruptInner) -> Self {
        ExecutionInterrupt {
            inner: Box::new(inner),
        }
    }

    pub(crate) fn into_outcome<T>(
        self,
        should_catch: impl FnOnce(&ControlFlowInterrupt) -> bool,
    ) -> ExecutionResult<ExecutionOutcome<T>> {
        match *self.inner {
            ExecutionInterruptInner::ControlFlow(control_flow_interrupt, _)
                if should_catch(&control_flow_interrupt) =>
            {
                Ok(ExecutionOutcome::ControlFlow(control_flow_interrupt))
            }
            _ => Err(self),
        }
    }

    fn into_execution_error<T>(self) -> ExecutionResult<Result<T, syn::Error>> {
        match *self.inner {
            ExecutionInterruptInner::Error(error) => Ok(Err(error)),
            _ => Err(self),
        }
    }

    pub(crate) fn parse_error(error: ParseError) -> Self {
        Self::new(ExecutionInterruptInner::ParseError(error))
    }

    pub(crate) fn error(error: syn::Error) -> Self {
        Self::new(ExecutionInterruptInner::Error(error))
    }

    pub(crate) fn control_flow(control_flow: ControlFlowInterrupt, span: Span) -> Self {
        Self::new(ExecutionInterruptInner::ControlFlow(control_flow, span))
    }
}

#[derive(Debug)]
enum ExecutionInterruptInner {
    Error(syn::Error),
    ParseError(ParseError),
    ControlFlow(ControlFlowInterrupt, Span),
}

#[derive(Debug)]
pub(crate) enum ControlFlowInterrupt {
    Break,
    Continue,
}

impl ControlFlowInterrupt {
    pub(crate) fn catch_any(_: &ControlFlowInterrupt) -> bool {
        true
    }
}

impl From<syn::Error> for ExecutionInterrupt {
    fn from(e: syn::Error) -> Self {
        ExecutionInterrupt::error(e)
    }
}

impl From<ParseError> for ExecutionInterrupt {
    fn from(e: ParseError) -> Self {
        ExecutionInterrupt::parse_error(e)
    }
}

impl ExecutionInterrupt {
    pub(crate) fn convert_to_final_error(self) -> syn::Error {
        match *self.inner {
            ExecutionInterruptInner::Error(e) => e,
            ExecutionInterruptInner::ParseError(e) => e.convert_to_final_error(),
            ExecutionInterruptInner::ControlFlow(ControlFlowInterrupt::Break, span) => {
                syn::Error::new(span, "Break can only be used inside a loop")
            }
            ExecutionInterruptInner::ControlFlow(ControlFlowInterrupt::Continue, span) => {
                syn::Error::new(span, "Continue can only be used inside a loop")
            }
        }
    }
}
