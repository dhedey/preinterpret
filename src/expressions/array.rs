use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionArray {
    pub(crate) items: Vec<ExpressionValue>,
    /// The span range that generated this value.
    /// For a complex expression, the start span is the most left part
    /// of the expression, and the end span is the most right part.
    pub(crate) span_range: SpanRange,
}

impl ExpressionArray {
    pub(super) fn handle_unary_operation(
        mut self,
        operation: OutputSpanned<UnaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        Ok(match operation.operation {
            UnaryOperation::Neg { .. } | UnaryOperation::Not { .. } => {
                return operation.unsupported(self)
            }
            UnaryOperation::Cast { target, target_ident, .. } => match target {
                CastTarget::Stream => {
                    operation.output(self.to_stream_with_grouped_items()?)
                },
                CastTarget::Group => {
                    operation.output(operation.output(self.to_stream_with_grouped_items()?).into_new_output_stream(Grouping::Grouped)?)
                },
                _ => {
                    if self.items.len() == 1 {
                        self.items.pop().unwrap().handle_unary_operation(operation)?
                    } else {
                        return operation.execution_err(format!(
                            "Cannot only attempt to cast a singleton array to {} but the array has {} elements",
                            target_ident,
                            self.items.len(),
                        ));
                    }
                }
            },
        })
    }

    fn to_stream_with_grouped_items(self) -> ExecutionResult<OutputStream> {
        let mut stream = OutputStream::new();
        for item in self.items {
            item.output_to(Grouping::Grouped, &mut stream)?;
        }
        Ok(stream)
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
        let lhs = self.items;
        let rhs = rhs.items;
        Ok(match operation.operation {
            PairedBinaryOperation::Addition { .. } => operation.output({
                let mut stream = lhs;
                stream.extend(rhs);
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

impl HasValueType for ExpressionArray {
    fn value_type(&self) -> &'static str {
        self.items.value_type()
    }
}

impl HasValueType for Vec<ExpressionValue> {
    fn value_type(&self) -> &'static str {
        "array"
    }
}

impl ToExpressionValue for Vec<ExpressionValue> {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        ExpressionValue::Array(ExpressionArray {
            items: self,
            span_range,
        })
    }
}
