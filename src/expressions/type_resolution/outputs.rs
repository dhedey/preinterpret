use super::*;

/// A [`ReturnedValue`] represents an arbitrary output from a method / expression.
/// It will then be mapped into a [`RequestedValue`] according to the requested ownership.
///
/// See also [`RequestedValue`] for values that have been fully evaluated.
pub(crate) enum ReturnedValue {
    Owned(AnyValueOwned),
    CopyOnWrite(CopyOnWriteValue),
    Mutable(AnyValueMutable),
    Shared(AnyValueShared),
}

// TODO: Find some way to selectively enable only on MSRV (e.g. following the build.rs feature flag pattern)
// #[diagnostic::on_unimplemented(
//     message = "`ResolvableOutput` is not implemented for `{Self}`",
//     note = "`ResolvableOutput` is not implemented for `Shared<X>` or `Mutable<X>` unless `X` is `Value`. If we wish to change this, we'd need to have some way to represent some kind of `ExpressionReference`, i.e. a `Typed<Shared<..>>` rather than a `Shared<Typed<..>>`"
// )]
pub(crate) trait IsReturnable {
    fn to_returned_value(self) -> FunctionResult<ReturnedValue>;
}

impl IsReturnable for ReturnedValue {
    fn to_returned_value(self) -> FunctionResult<ReturnedValue> {
        Ok(self)
    }
}

impl IsReturnable for AnyValueShared {
    fn to_returned_value(self) -> FunctionResult<ReturnedValue> {
        Ok(ReturnedValue::Shared(self))
    }
}

impl IsReturnable for AnyValueMutable {
    fn to_returned_value(self) -> FunctionResult<ReturnedValue> {
        Ok(ReturnedValue::Mutable(self))
    }
}

impl<T: IsReturnable> IsReturnable for FunctionResult<T> {
    fn to_returned_value(self) -> FunctionResult<ReturnedValue> {
        self?.to_returned_value()
    }
}
