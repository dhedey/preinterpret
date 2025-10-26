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
            ExecutionInterruptInner::ControlFlowInterrupt(control_flow_interrupt, _)
                if should_catch(&control_flow_interrupt) =>
            {
                Ok(ExecutionOutcome::ControlFlow(control_flow_interrupt))
            }
            _ => Err(self),
        }
    }

    /// Determines which errors can be caught by an attempt block.
    ///
    /// Generally, coding errors should be propogated, while user-thrown errors
    /// and runtime errors which are indicative of invalid values being
    /// present should be caught.
    ///
    /// This allows the attempt block to use the first valid branch given the data
    /// it encounters.
    pub(crate) fn is_catchable_error(&self) -> bool {
        match self.inner.as_ref() {
            ExecutionInterruptInner::SyntaxError { .. } => false,
            ExecutionInterruptInner::TypeError { .. } => false,
            ExecutionInterruptInner::OwnershipError { .. } => false,
            ExecutionInterruptInner::DebugError { .. } => false,
            ExecutionInterruptInner::AssertionError { .. } => true,
            ExecutionInterruptInner::ValueError { .. } => true,
            ExecutionInterruptInner::ControlFlowError { .. } => false,
            ExecutionInterruptInner::RuntimeParseError { .. } => true,
            ExecutionInterruptInner::ControlFlowInterrupt { .. } => false,
        }
    }

    pub(crate) fn syntax_error(error: syn::Error) -> Self {
        Self::new(ExecutionInterruptInner::SyntaxError(error))
    }

    pub(crate) fn type_error(error: syn::Error) -> Self {
        Self::new(ExecutionInterruptInner::TypeError(error))
    }

    pub(crate) fn ownership_error(error: syn::Error) -> Self {
        Self::new(ExecutionInterruptInner::OwnershipError(error))
    }

    pub(crate) fn debug_error(error: syn::Error) -> Self {
        Self::new(ExecutionInterruptInner::DebugError(error))
    }

    pub(crate) fn assertion_error(error: syn::Error) -> Self {
        Self::new(ExecutionInterruptInner::AssertionError(error))
    }

    pub(crate) fn runtime_parse_error(error: ParseError) -> Self {
        Self::new(ExecutionInterruptInner::RuntimeParseError(error))
    }

    pub(crate) fn value_error(error: syn::Error) -> Self {
        Self::new(ExecutionInterruptInner::ValueError(error))
    }

    pub(crate) fn control_flow_error(error: syn::Error) -> Self {
        Self::new(ExecutionInterruptInner::ControlFlowError(error))
    }

    pub(crate) fn control_flow(control_flow: ControlFlowInterrupt, span: Span) -> Self {
        Self::new(ExecutionInterruptInner::ControlFlowInterrupt(
            control_flow,
            span,
        ))
    }
}

impl From<ParseError> for ExecutionInterrupt {
    fn from(e: ParseError) -> Self {
        ExecutionInterrupt::runtime_parse_error(e)
    }
}

#[derive(Debug)]
enum ExecutionInterruptInner {
    /// Some error with preinterpet syntax
    SyntaxError(syn::Error),
    /// Method doesn't exist on value, etc
    TypeError(syn::Error),
    /// Some violation of borrowing rules or unique ownership
    OwnershipError(syn::Error),
    /// An error from `.debug()` which shouldn't be caught
    DebugError(syn::Error),
    /// User-thrown errors
    AssertionError(syn::Error),
    /// An unexpected value (e.g. out-of-bounds index)
    ValueError(syn::Error),
    /// An error caused by invalid control flow (e.g. no matching attempt arm)
    ControlFlowError(syn::Error),
    /// A parse error which occurred during runtime
    /// (e.g. from parsing macro arguments in a preinterpret parser)
    RuntimeParseError(ParseError),
    /// Indicates unwinding due to control flow (break/continue)
    ControlFlowInterrupt(ControlFlowInterrupt, Span),
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

impl ExecutionInterrupt {
    pub(crate) fn convert_to_final_error(self) -> syn::Error {
        match *self.inner {
            ExecutionInterruptInner::SyntaxError(error) => error,
            ExecutionInterruptInner::TypeError(error) => error,
            ExecutionInterruptInner::OwnershipError(error) => error,
            ExecutionInterruptInner::DebugError(error) => error,
            ExecutionInterruptInner::AssertionError(error) => error,
            ExecutionInterruptInner::ValueError(error) => error,
            ExecutionInterruptInner::ControlFlowError(error) => error,
            ExecutionInterruptInner::RuntimeParseError(e) => e.convert_to_final_error(),
            ExecutionInterruptInner::ControlFlowInterrupt(ControlFlowInterrupt::Break, span) => {
                syn::Error::new(span, "Break can only be used inside a loop")
            }
            ExecutionInterruptInner::ControlFlowInterrupt(ControlFlowInterrupt::Continue, span) => {
                syn::Error::new(span, "Continue can only be used inside a loop")
            }
        }
    }
}
