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
        self.map_err(|error| error.convert_to_syn_error())
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
    pub(crate) fn convert_to_syn_error(self) -> syn::Error {
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

    pub(crate) fn add_context_if_none(self, context: impl std::fmt::Display) -> Self {
        ParseError(self.0.add_context_if_none(context))
    }

    /// This is not a `From` because it wants to be explicit
    pub(crate) fn convert_to_syn_error(self) -> syn::Error {
        self.0.convert_to_syn_error()
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
    /// Asserts that the result contains no control flow interrupts (only errors).
    /// This is used at boundaries where control flow should have already been caught.
    fn expect_no_interrupts(self) -> FunctionResult<T>;
}

impl<T> ExecutionResultExt<T> for ExecutionResult<T> {
    fn expect_no_interrupts(self) -> FunctionResult<T> {
        self.map_err(FunctionError::new)
    }
}

/// A result type for functions that can produce errors but NOT control-flow interrupts.
pub(crate) type FunctionResult<T> = core::result::Result<T, FunctionError>;

/// A newtype wrapping `ExecutionInterrupt` that asserts it is not a control flow interrupt.
/// This is used for functions that cannot produce control flow (break/continue/revert).
#[derive(Debug)]
pub(crate) struct FunctionError(ExecutionInterrupt);

impl FunctionError {
    pub(crate) fn new(interrupt: ExecutionInterrupt) -> Self {
        debug_assert!(
            !matches!(
                interrupt.inner.as_ref(),
                ExecutionInterruptInner::ControlFlowInterrupt(_)
            ),
            "FunctionError should not wrap a control flow interrupt. \
             Please report this bug at https://github.com/dhedey/preinterpret/issues"
        );
        FunctionError(interrupt)
    }

    /// Determines if the error can be caught when attempting to map a late-bound
    /// mutable value, to retry as a shared value instead.
    pub(crate) fn into_caught_mutable_map_attempt_error(self) -> Result<syn::Error, Self> {
        match self.0.inner.as_ref() {
            ExecutionInterruptInner::Error(ExecutionError(ErrorKind::Value, _)) => {
                Ok(self.0.expect_error().convert_to_syn_error())
            }
            _ => Err(self),
        }
    }
}

pub(crate) trait FunctionResultExt<T> {
    fn into_execution_result(self) -> ExecutionResult<T>;

    /// This is not a `From` because it wants to be explicit
    fn convert_to_final_result(self) -> syn::Result<T>;
}

impl<T> FunctionResultExt<T> for FunctionResult<T> {
    fn into_execution_result(self) -> ExecutionResult<T> {
        self.map_err(|error| error.into())
    }

    fn convert_to_final_result(self) -> syn::Result<T> {
        self.map_err(|error| error.0.expect_error().convert_to_syn_error())
    }
}

impl From<FunctionError> for ExecutionInterrupt {
    fn from(error: FunctionError) -> Self {
        error.0
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
        let error = ExecutionError(kind, DetailedError::Standard(error));
        ExecutionInterrupt {
            inner: Box::new(ExecutionInterruptInner::Error(error)),
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
            ExecutionInterruptInner::Error(ExecutionError(ErrorKind::Syntax, _)) => false,
            ExecutionInterruptInner::Error(ExecutionError(ErrorKind::Type, _)) => false,
            ExecutionInterruptInner::Error(ExecutionError(ErrorKind::Ownership, _)) => false,
            ExecutionInterruptInner::Error(ExecutionError(ErrorKind::Debug, _)) => false,
            ExecutionInterruptInner::Error(ExecutionError(ErrorKind::Assertion, _)) => true,
            ExecutionInterruptInner::Error(ExecutionError(ErrorKind::Value, _)) => true,
            ExecutionInterruptInner::Error(ExecutionError(ErrorKind::ControlFlow, _)) => false,
            ExecutionInterruptInner::Error(ExecutionError(ErrorKind::Parse, _)) => true,
            ExecutionInterruptInner::ControlFlowInterrupt(interrupt) => {
                interrupt.catch_location_id() == catch_location_id
            }
        }
    }

    pub(crate) fn error_mut(&mut self) -> Option<(ErrorKind, &mut DetailedError)> {
        Some(match self.inner.as_mut() {
            ExecutionInterruptInner::Error(ExecutionError(kind, error)) => (*kind, error),
            ExecutionInterruptInner::ControlFlowInterrupt { .. } => return None,
        })
    }

    pub(crate) fn ownership_error(error: syn::Error) -> Self {
        Self::new_error(ErrorKind::Ownership, error)
    }

    pub(crate) fn parse_error(error: ParseError) -> Self {
        Self::new(ExecutionInterruptInner::Error(ExecutionError(
            ErrorKind::Parse,
            error.0,
        )))
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

impl From<ParseError> for FunctionError {
    fn from(e: ParseError) -> Self {
        FunctionError::new(ExecutionInterrupt::parse_error(e))
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
    Error(ExecutionError),
    /// Indicates unwinding due to control flow (break/continue)
    ControlFlowInterrupt(ControlFlowInterrupt),
}

#[derive(Debug)]
pub(crate) struct ExecutionError(ErrorKind, DetailedError);

impl ExecutionError {
    pub(crate) fn new(kind: ErrorKind, error: syn::Error) -> Self {
        ExecutionError(kind, DetailedError::Standard(error))
    }

    pub(crate) fn convert_to_syn_error(self) -> syn::Error {
        self.1.convert_to_syn_error()
    }
}

impl From<ExecutionError> for ExecutionInterrupt {
    fn from(error: ExecutionError) -> Self {
        ExecutionInterrupt::new(ExecutionInterruptInner::Error(error))
    }
}

impl From<ExecutionError> for FunctionError {
    fn from(error: ExecutionError) -> Self {
        FunctionError(ExecutionInterrupt::from(error))
    }
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
        value: Option<AnyValue>,
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
    value: Option<AnyValue>,
}

impl BreakInterrupt {
    pub(crate) fn into_requested_value(
        self,
        span_range: SpanRange,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue> {
        let value = match self.value {
            Some(value) => value,
            None => ().into_any_value(),
        };
        Ok(ownership.map_from_owned(Spanned(value, span_range))?.0)
    }
}

pub(crate) struct ContinueInterrupt {
    target_catch_location: CatchLocationId,
}

pub(crate) struct RevertInterrupt {
    target_catch_location: CatchLocationId,
}

impl ExecutionInterrupt {
    pub(crate) fn expect_error(self) -> ExecutionError {
        match *self.inner {
            ExecutionInterruptInner::Error(error) => error,
            ExecutionInterruptInner::ControlFlowInterrupt(_) => panic!(
                "Internal error: expected error but got control flow interrupt. \
                 Please report this bug at https://github.com/dhedey/preinterpret/issues"
            ),
        }
    }
}
