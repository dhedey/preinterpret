use super::*;

/// A [`ReturnedValue`] represents an arbitrary output from a method / expression.
/// It will then be mapped into a [`RequestedValue`] according to the requested ownership.
///
/// See also [`RequestedValue`] for values that have been fully evaluated.
pub(crate) enum ReturnedValue {
    Owned(OwnedValue),
    CopyOnWrite(CopyOnWriteValue),
    Mutable(MutableValue),
    Shared(SharedValue),
}

impl WithSpanRangeExt for ReturnedValue {
    fn with_span_range(self, span_range: SpanRange) -> Self {
        match self {
            ReturnedValue::Owned(v) => ReturnedValue::Owned(v.with_span_range(span_range)),
            ReturnedValue::CopyOnWrite(v) => {
                ReturnedValue::CopyOnWrite(v.with_span_range(span_range))
            }
            ReturnedValue::Mutable(v) => ReturnedValue::Mutable(v.with_span_range(span_range)),
            ReturnedValue::Shared(v) => ReturnedValue::Shared(v.with_span_range(span_range)),
        }
    }
}

// TODO: Find some way to selectively enable only on MSRV (e.g. following the build.rs feature flag pattern)
// #[diagnostic::on_unimplemented(
//     message = "`ResolvableOutput` is not implemented for `{Self}`",
//     note = "`ResolvableOutput` is not implemented for `Shared<X>` or `Mutable<X>` unless `X` is `Value`. If we wish to change this, we'd need to have some way to represent some kind of `ExpressionReference`, i.e. a `Typed<Shared<..>>` rather than a `Shared<Typed<..>>`"
// )]
pub(crate) trait IsReturnable {
    fn to_returned_value(self, output_span_range: SpanRange) -> ExecutionResult<ReturnedValue>;
}

impl IsReturnable for ReturnedValue {
    fn to_returned_value(self, output_span_range: SpanRange) -> ExecutionResult<ReturnedValue> {
        Ok(self.with_span_range(output_span_range))
    }
}

impl IsReturnable for SharedValue {
    fn to_returned_value(self, output_span_range: SpanRange) -> ExecutionResult<ReturnedValue> {
        Ok(ReturnedValue::Shared(
            self.update_span_range(|_| output_span_range),
        ))
    }
}

impl IsReturnable for MutableValue {
    fn to_returned_value(self, output_span_range: SpanRange) -> ExecutionResult<ReturnedValue> {
        Ok(ReturnedValue::Mutable(
            self.update_span_range(|_| output_span_range),
        ))
    }
}

impl<T: IntoValue> IsReturnable for T {
    fn to_returned_value(self, output_span_range: SpanRange) -> ExecutionResult<ReturnedValue> {
        Ok(ReturnedValue::Owned(
            self.into_owned_value(output_span_range),
        ))
    }
}

impl<T: IntoValue> IsReturnable for Spanned<Owned<T>> {
    fn to_returned_value(self, output_span_range: SpanRange) -> ExecutionResult<ReturnedValue> {
        Ok(ReturnedValue::Owned(
            self.map_owned(|f, _| f.into_value())
                .with_span_range(output_span_range),
        ))
    }
}

impl<T: IsReturnable> IsReturnable for ExecutionResult<T> {
    fn to_returned_value(self, output_span_range: SpanRange) -> ExecutionResult<ReturnedValue> {
        self?.to_returned_value(output_span_range)
    }
}

pub(crate) trait StreamAppender {
    fn append(self, output: &mut OutputStream) -> ExecutionResult<()>;
}

impl<F: FnOnce(&mut OutputStream) -> ExecutionResult<()>> StreamAppender for F {
    fn append(self, output: &mut OutputStream) -> ExecutionResult<()> {
        self(output)
    }
}

pub(crate) struct StreamOutput<T: StreamAppender>(T);
impl<F: FnOnce(&mut OutputStream) -> ExecutionResult<()>> StreamOutput<F> {
    pub fn new(appender: F) -> Self {
        Self(appender)
    }
}
impl<T: StreamAppender> From<T> for StreamOutput<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
impl<T: StreamAppender> IsReturnable for StreamOutput<T> {
    fn to_returned_value(self, output_span_range: SpanRange) -> ExecutionResult<ReturnedValue> {
        let mut output = OutputStream::new();
        self.0.append(&mut output)?;
        output.to_returned_value(output_span_range)
    }
}
