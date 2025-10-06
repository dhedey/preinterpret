use super::*;

// TODO: Find some way to selectively enable only on MSRV (e.g. following the build.rs feature flag pattern)
// #[diagnostic::on_unimplemented(
//     message = "`ResolvableOutput` is not implemented for `{Self}`",
//     note = "`ResolvableOutput` is not implemented for `Shared<X>` or `Mutable<X>` unless `X` is `ExpressionValue`. If we wish to change this, we'd need to have some way to represent some kind of `ExpressionReference`, i.e. a `Typed<Shared<..>>` rather than a `Shared<Typed<..>>`"
// )]
pub(crate) trait ResolvableOutput {
    fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue>;
}

impl ResolvableOutput for ResolvedValue {
    fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
        Ok(self.with_span_range(output_span_range))
    }
}

impl ResolvableOutput for Shared<ExpressionValue> {
    fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
        Ok(ResolvedValue::Shared(
            self.update_span_range(|_| output_span_range),
        ))
    }
}

impl ResolvableOutput for Mutable<ExpressionValue> {
    fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
        Ok(ResolvedValue::Mutable(
            self.update_span_range(|_| output_span_range),
        ))
    }
}

impl<T: ToExpressionValue> ResolvableOutput for T {
    fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
        Ok(ResolvedValue::Owned(
            self.into_owned_value(output_span_range),
        ))
    }
}

impl<T: ToExpressionValue> ResolvableOutput for Owned<T> {
    fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
        Ok(ResolvedValue::Owned(
            self.map(|f, _| f.into_value())
                .with_span_range(output_span_range),
        ))
    }
}

impl<T: ResolvableOutput> ResolvableOutput for ExecutionResult<T> {
    fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
        self?.to_resolved_value(output_span_range)
    }
}

impl ResolvableOutput for Ident {
    fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
        OutputStream::new_with(|s| s.push_ident(self)).to_resolved_value(output_span_range)
    }
}

impl ResolvableOutput for Literal {
    fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
        ExpressionValue::for_literal(self).to_resolved_value(output_span_range)
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
impl<T: StreamAppender> ResolvableOutput for StreamOutput<T> {
    fn to_resolved_value(self, output_span_range: SpanRange) -> ExecutionResult<ResolvedValue> {
        let mut output = OutputStream::new();
        self.0.append(&mut output)?;
        output.to_resolved_value(output_span_range)
    }
}
