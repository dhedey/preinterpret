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
pub(crate) enum DetailedError {
    Standard(syn::Error),
    Contextual(syn::Error, String),
}

impl Default for DetailedError {
    fn default() -> Self {
        DetailedError::Standard(syn::Error::new(
            proc_macro2::Span::call_site(),
            "An unknown error occurred",
        ))
    }
}

impl HasSpan for DetailedError {
    fn span(&self) -> Span {
        match self {
            DetailedError::Standard(e) => e.span(),
            DetailedError::Contextual(e, _) => e.span(),
        }
    }
}

impl DetailedError {
    /// This is not a `From` because it wants to be explicit
    pub(crate) fn convert_to_final_error(self) -> syn::Error {
        match self {
            DetailedError::Standard(e) => e,
            DetailedError::Contextual(e, message) => e.concat(&format!("\n{}", message)),
        }
    }

    pub(crate) fn add_context_if_none(self, context: impl std::fmt::Display) -> Self {
        match self {
            DetailedError::Standard(e) => DetailedError::Contextual(e, context.to_string()),
            other => other,
        }
    }

    pub(crate) fn context(&self) -> Option<&str> {
        match self {
            DetailedError::Standard(_) => None,
            DetailedError::Contextual(_, context) => Some(context),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ParseError(DetailedError);

impl From<syn::Error> for ParseError {
    fn from(e: syn::Error) -> Self {
        ParseError(DetailedError::Standard(e))
    }
}

impl HasSpan for ParseError {
    fn span(&self) -> Span {
        self.0.span()
    }
}

impl ParseError {
    pub(crate) fn new(error: syn::Error) -> Self {
        ParseError(DetailedError::Standard(error))
    }

    /// This is not a `From` because it wants to be explicit
    pub(crate) fn convert_to_final_error(self) -> syn::Error {
        self.0.convert_to_final_error()
    }

    pub(crate) fn add_context_if_none(self, context: impl std::fmt::Display) -> Self {
        Self(self.0.add_context_if_none(context))
    }

    pub(crate) fn context(&self) -> Option<&str> {
        self.0.context()
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

    fn new_error(kind: ErrorKind, error: syn::Error) -> Self {
        ExecutionInterrupt {
            inner: Box::new(ExecutionInterruptInner::Error(
                kind,
                DetailedError::Standard(error),
            )),
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
            ExecutionInterruptInner::Error(ErrorKind::Syntax, _) => false,
            ExecutionInterruptInner::Error(ErrorKind::Type, _) => false,
            ExecutionInterruptInner::Error(ErrorKind::Ownership, _) => false,
            ExecutionInterruptInner::Error(ErrorKind::Debug, _) => false,
            ExecutionInterruptInner::Error(ErrorKind::Assertion, _) => true,
            ExecutionInterruptInner::Error(ErrorKind::Value, _) => true,
            ExecutionInterruptInner::Error(ErrorKind::ControlFlow, _) => false,
            ExecutionInterruptInner::Error(ErrorKind::Parse, _) => true,
            ExecutionInterruptInner::ControlFlowInterrupt { .. } => false,
        }
    }

    pub(crate) fn error_mut(&mut self) -> Option<(ErrorKind, &mut DetailedError)> {
        Some(match self.inner.as_mut() {
            ExecutionInterruptInner::Error(kind, error) => (*kind, error),
            ExecutionInterruptInner::ControlFlowInterrupt { .. } => return None,
        })
    }

    pub(crate) fn syntax_error(error: syn::Error) -> Self {
        Self::new_error(ErrorKind::Syntax, error)
    }

    pub(crate) fn type_error(error: syn::Error) -> Self {
        Self::new_error(ErrorKind::Type, error)
    }

    pub(crate) fn ownership_error(error: syn::Error) -> Self {
        Self::new_error(ErrorKind::Ownership, error)
    }

    pub(crate) fn debug_error(error: syn::Error) -> Self {
        Self::new_error(ErrorKind::Debug, error)
    }

    pub(crate) fn assertion_error(error: syn::Error) -> Self {
        Self::new_error(ErrorKind::Assertion, error)
    }

    pub(crate) fn parse_error(error: ParseError) -> Self {
        Self::new(ExecutionInterruptInner::Error(ErrorKind::Parse, error.0))
    }

    pub(crate) fn value_error(error: syn::Error) -> Self {
        Self::new_error(ErrorKind::Value, error)
    }

    pub(crate) fn control_flow_error(error: syn::Error) -> Self {
        Self::new_error(ErrorKind::ControlFlow, error)
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
        ExecutionInterrupt::parse_error(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ErrorKind {
    /// Some error with preinterpet syntax
    Syntax,
    /// Method doesn't exist on value, etc
    Type,
    /// Some violation of borrowing rules or unique ownership
    Ownership,
    /// An error from `.debug()` which shouldn't be caught
    Debug,
    /// User-thrown errors
    Assertion,
    /// An unexpected value (e.g. out-of-bounds index)
    Value,
    /// An error caused by invalid control flow (e.g. no matching attempt arm)
    ControlFlow,
    /// A parse error which occurred during runtime
    /// (e.g. from parsing macro arguments in a preinterpret parser)
    Parse,
}

impl ErrorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            ErrorKind::Syntax => "SyntaxError",
            ErrorKind::Type => "TypeError",
            ErrorKind::Ownership => "OwnershipError",
            ErrorKind::Debug => "DebugError",
            ErrorKind::Assertion => "AssertionError",
            ErrorKind::Value => "ValueError",
            ErrorKind::ControlFlow => "ControlFlowError",
            ErrorKind::Parse => "ParseError",
        }
    }
}

#[derive(Debug)]
enum ExecutionInterruptInner {
    /// Some runtime error
    Error(ErrorKind, DetailedError),
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
            ExecutionInterruptInner::Error(_, e) => e.convert_to_final_error(),
            ExecutionInterruptInner::ControlFlowInterrupt(ControlFlowInterrupt::Break, span) => {
                syn::Error::new(span, "Break can only be used inside a loop")
            }
            ExecutionInterruptInner::ControlFlowInterrupt(ControlFlowInterrupt::Continue, span) => {
                syn::Error::new(span, "Continue can only be used inside a loop")
            }
        }
    }
}
