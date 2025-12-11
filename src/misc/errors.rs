use crate::internal_prelude::*;

pub(crate) type ParseResult<T> = core::result::Result<T, ParseError>;

#[allow(unused)]
pub(crate) trait ParseResultExt<T> {
    /// This is not a `From` because it wants to be explicit
    fn convert_to_final_result(self) -> syn::Result<T>;
    fn into_execution_result(self) -> ExecutionResult<T>;
}

impl<T> ParseResultExt<T> for ParseResult<T> {
    fn convert_to_final_result(self) -> syn::Result<T> {
        self.map_err(|error| error.convert_to_final_error())
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
}

// Ideally this would be our own enum with Completed / Interrupted variants,
// but we want it to work with `?` and defining custom FromResidual is not
// possible on stable (at least according to our MSRV).
pub(crate) type ExecutionResult<T> = core::result::Result<T, ExecutionInterrupt>;

pub(crate) enum ExecutionOutcome<T> {
    Value(Spanned<T>),
    ControlFlow(ControlFlowInterrupt),
}

pub(crate) trait ExecutionResultExt<T> {
    /// This is not a `From` because it wants to be explicit
    fn convert_to_final_result(self) -> syn::Result<T>;
}

impl<T> ExecutionResultExt<T> for ExecutionResult<T> {
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
        catch_location_id: CatchLocationId,
    ) -> ExecutionResult<ExecutionOutcome<T>> {
        match *self.inner {
            ExecutionInterruptInner::ControlFlowInterrupt(interrupt)
                if catch_location_id == interrupt.catch_location_id() =>
            {
                Ok(ExecutionOutcome::ControlFlow(interrupt))
            }
            _ => Err(self),
        }
    }

    /// Generally, coding errors should be propagated, while user-thrown errors
    /// and runtime errors which are indicative of invalid values being
    /// present should be caught.
    ///
    /// This allows the attempt block to use the first valid branch given the data
    /// it encounters.
    pub(crate) fn is_catchable_by_attempt_block(&self, catch_location_id: CatchLocationId) -> bool {
        match self.inner.as_ref() {
            ExecutionInterruptInner::Error(ErrorKind::Syntax, _) => false,
            ExecutionInterruptInner::Error(ErrorKind::Type, _) => false,
            ExecutionInterruptInner::Error(ErrorKind::Ownership, _) => false,
            ExecutionInterruptInner::Error(ErrorKind::Debug, _) => false,
            ExecutionInterruptInner::Error(ErrorKind::Assertion, _) => true,
            ExecutionInterruptInner::Error(ErrorKind::Value, _) => true,
            ExecutionInterruptInner::Error(ErrorKind::ControlFlow, _) => false,
            ExecutionInterruptInner::Error(ErrorKind::Parse, _) => true,
            ExecutionInterruptInner::ControlFlowInterrupt(interrupt) => {
                interrupt.catch_location_id() == catch_location_id
            }
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

    pub(crate) fn control_flow(control_flow: ControlFlowInterrupt) -> Self {
        Self::new(ExecutionInterruptInner::ControlFlowInterrupt(control_flow))
    }
}

impl From<ParseError> for ExecutionInterrupt {
    fn from(e: ParseError) -> Self {
        ExecutionInterrupt::parse_error(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ErrorKind {
    /// Some error with preinterpret syntax
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
    ControlFlowInterrupt(ControlFlowInterrupt),
}

pub(crate) enum ControlFlowInterrupt {
    Break(BreakInterrupt),
    Continue(ContinueInterrupt),
    Revert(RevertInterrupt),
}

impl std::fmt::Debug for ControlFlowInterrupt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlFlowInterrupt::Break(_) => f.write_str("Break"),
            ControlFlowInterrupt::Continue(_) => f.write_str("Continue"),
            ControlFlowInterrupt::Revert(_) => f.write_str("Revert"),
        }
    }
}

impl ControlFlowInterrupt {
    pub(crate) fn new_break(
        target_catch_location: CatchLocationId,
        value: Option<OwnedValue>,
    ) -> Self {
        ControlFlowInterrupt::Break(BreakInterrupt {
            target_catch_location,
            value,
        })
    }

    pub(crate) fn new_continue(target_catch_location: CatchLocationId) -> Self {
        ControlFlowInterrupt::Continue(ContinueInterrupt {
            target_catch_location,
        })
    }

    pub(crate) fn new_revert(target_catch_location: CatchLocationId) -> Self {
        ControlFlowInterrupt::Revert(RevertInterrupt {
            target_catch_location,
        })
    }

    fn catch_location_id(&self) -> CatchLocationId {
        match self {
            ControlFlowInterrupt::Break(break_interrupt) => break_interrupt.target_catch_location,
            ControlFlowInterrupt::Continue(continue_interrupt) => {
                continue_interrupt.target_catch_location
            }
            ControlFlowInterrupt::Revert(revert_interrupt) => {
                revert_interrupt.target_catch_location
            }
        }
    }
}

pub(crate) struct BreakInterrupt {
    target_catch_location: CatchLocationId,
    value: Option<OwnedValue>,
}

impl BreakInterrupt {
    pub(crate) fn into_value(
        self,
        span_range: SpanRange,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue> {
        let value = match self.value {
            Some(value) => value,
            None => ().into_owned_value(),
        };
        ownership
            .map_from_owned(Spanned(value, span_range))
            .map(|spanned| spanned.0)
    }
}

pub(crate) struct ContinueInterrupt {
    target_catch_location: CatchLocationId,
}

pub(crate) struct RevertInterrupt {
    target_catch_location: CatchLocationId,
}

impl ExecutionInterrupt {
    pub(crate) fn convert_to_final_error(self) -> syn::Error {
        match *self.inner {
            ExecutionInterruptInner::Error(_, e) => e.convert_to_final_error(),
            ExecutionInterruptInner::ControlFlowInterrupt(ControlFlowInterrupt::Break(_)) => {
                panic!(
                    "Internal error: break escaped to root (should be caught at parse time). \
                     Please report this bug at https://github.com/dhedey/preinterpret/issues"
                )
            }
            ExecutionInterruptInner::ControlFlowInterrupt(ControlFlowInterrupt::Continue(_)) => {
                panic!(
                    "Internal error: continue escaped to root (should be caught at parse time). \
                     Please report this bug at https://github.com/dhedey/preinterpret/issues"
                )
            }
            ExecutionInterruptInner::ControlFlowInterrupt(ControlFlowInterrupt::Revert(_)) => {
                panic!(
                    "Internal error: revert escaped to root (should be caught at parse time). \
                     Please report this bug at https://github.com/dhedey/preinterpret/issues"
                )
            }
        }
    }
}
