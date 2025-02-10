use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionStream {
    pub(super) value: OutputStream,
    /// The span range that generated this value.
    /// For a complex expression, the start span is the most left part
    /// of the expression, and the end span is the most right part.
    pub(super) span_range: SpanRange,
}

impl ExpressionStream {
    pub(super) fn handle_unary_operation(
        self,
        operation: OutputSpanned<UnaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        Ok(match operation.operation {
            UnaryOperation::Neg { .. } | UnaryOperation::Not { .. } => {
                return operation.unsupported(self)
            }
            UnaryOperation::Cast { target, .. } => match target {
                CastTarget::Stream => operation.output(self.value),
                _ => {
                    let coerced = self.value.coerce_into_value(self.span_range);
                    if let ExpressionValue::Stream(_) = &coerced {
                        return operation.unsupported(coerced);
                    }
                    coerced.handle_unary_operation(operation)?
                }
            },
        })
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        _right: ExpressionInteger,
        operation: OutputSpanned<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        operation.unsupported(self)
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: OutputSpanned<PairedBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        let lhs = self.value;
        let rhs = rhs.value;
        Ok(match operation.operation {
            PairedBinaryOperation::Addition { .. } => operation.output({
                let mut stream = lhs;
                rhs.append_cloned_into(&mut stream);
                stream
            }),
            PairedBinaryOperation::Subtraction { .. }
            | PairedBinaryOperation::Multiplication { .. }
            | PairedBinaryOperation::Division { .. }
            | PairedBinaryOperation::LogicalAnd { .. }
            | PairedBinaryOperation::LogicalOr { .. }
            | PairedBinaryOperation::Remainder { .. }
            | PairedBinaryOperation::BitXor { .. }
            | PairedBinaryOperation::BitAnd { .. }
            | PairedBinaryOperation::BitOr { .. }
            | PairedBinaryOperation::Equal { .. }
            | PairedBinaryOperation::LessThan { .. }
            | PairedBinaryOperation::LessThanOrEqual { .. }
            | PairedBinaryOperation::NotEqual { .. }
            | PairedBinaryOperation::GreaterThanOrEqual { .. }
            | PairedBinaryOperation::GreaterThan { .. } => return operation.unsupported(lhs),
        })
    }
}

impl HasValueType for ExpressionStream {
    fn value_type(&self) -> &'static str {
        self.value.value_type()
    }
}

impl HasValueType for OutputStream {
    fn value_type(&self) -> &'static str {
        "stream"
    }
}

impl ToExpressionValue for OutputStream {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        ExpressionValue::Stream(ExpressionStream {
            value: self,
            span_range,
        })
    }
}
