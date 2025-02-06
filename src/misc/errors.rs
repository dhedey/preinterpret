use crate::internal_prelude::*;

pub(crate) type ParseResult<T> = core::result::Result<T, ParseError>;

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

pub(crate) trait ExecutionResultExt<T> {
    /// This is not a `From` because it wants to be explicit
    fn convert_to_final_result(self) -> syn::Result<T>;
}

impl<T> ExecutionResultExt<T> for ExecutionResult<T> {
    fn convert_to_final_result(self) -> syn::Result<T> {
        self.map_err(|error| error.convert_to_final_error())
    }
}

pub(crate) enum ExecutionInterrupt {
    Error(syn::Error),
    DestructureError(ParseError),
    ControlFlow(ControlFlowInterrupt, Span),
}

pub(crate) enum ControlFlowInterrupt {
    Break,
    Continue,
}

impl From<syn::Error> for ExecutionInterrupt {
    fn from(e: syn::Error) -> Self {
        ExecutionInterrupt::Error(e)
    }
}

impl From<ParseError> for ExecutionInterrupt {
    fn from(e: ParseError) -> Self {
        ExecutionInterrupt::DestructureError(e)
    }
}

impl ExecutionInterrupt {
    pub(crate) fn convert_to_final_error(self) -> syn::Error {
        match self {
            ExecutionInterrupt::Error(e) => e,
            ExecutionInterrupt::DestructureError(e) => e.convert_to_final_error(),
            ExecutionInterrupt::ControlFlow(ControlFlowInterrupt::Break, span) => {
                syn::Error::new(span, "Break can only be used inside a loop")
            }
            ExecutionInterrupt::ControlFlow(ControlFlowInterrupt::Continue, span) => {
                syn::Error::new(span, "Continue can only be used inside a loop")
            }
        }
    }
}
