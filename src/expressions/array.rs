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
            UnaryOperation::Cast {
                target,
                target_ident,
                ..
            } => match target {
                CastTarget::Stream => operation.output(self.stream_with_grouped_items()?),
                CastTarget::Group => operation.output(
                    operation
                        .output(self.stream_with_grouped_items()?)
                        .into_new_output_stream(
                            Grouping::Grouped,
                            StreamOutputBehaviour::Standard,
                        )?,
                ),
                CastTarget::String => operation.output({
                    let mut output = String::new();
                    self.concat_recursive_into(&mut output, &ConcatBehaviour::standard())?;
                    output
                }),
                CastTarget::DebugString => {
                    operation.output(self.items).into_debug_string_value()?
                }
                CastTarget::Boolean
                | CastTarget::Char
                | CastTarget::Integer(_)
                | CastTarget::Float(_) => {
                    if self.items.len() == 1 {
                        self.items
                            .pop()
                            .unwrap()
                            .handle_unary_operation(operation)?
                    } else {
                        return operation.execution_err(format!(
                            "Only a singleton array can be cast to {} but the array has {} elements",
                            target_ident,
                            self.items.len(),
                        ));
                    }
                }
            },
        })
    }

    fn stream_with_grouped_items(&self) -> ExecutionResult<OutputStream> {
        let mut output = OutputStream::new();
        self.output_grouped_items_to(&mut output)?;
        Ok(output)
    }

    pub(crate) fn output_grouped_items_to(&self, output: &mut OutputStream) -> ExecutionResult<()> {
        for item in &self.items {
            item.output_to(
                Grouping::Grouped,
                output,
                StreamOutputBehaviour::PermitArrays,
            )?;
        }
        Ok(())
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

    pub(super) fn into_indexed(
        mut self,
        access: IndexAccess,
        index: &ExpressionValue,
    ) -> ExecutionResult<ExpressionValue> {
        let span_range = SpanRange::new_between(self.span_range, access);
        Ok(match index {
            ExpressionValue::Integer(integer) => {
                let index = self.resolve_valid_index_from_integer(integer, false)?;
                self.items[index].clone().with_span_range(span_range)
            }
            ExpressionValue::Range(range) => {
                let range = range.resolve_to_index_range(&self)?;
                let new_items: Vec<_> = self.items.drain(range).collect();
                new_items.to_value(span_range)
            }
            _ => return index.execution_err("The index must be an integer or a range"),
        })
    }

    pub(super) fn index_mut(
        &mut self,
        _access: IndexAccess,
        index: &ExpressionValue,
    ) -> ExecutionResult<&mut ExpressionValue> {
        let index = self.resolve_valid_index(index, false)?;
        Ok(&mut self.items[index])
    }

    pub(super) fn index_ref(
        &self,
        _access: IndexAccess,
        index: &ExpressionValue,
    ) -> ExecutionResult<&ExpressionValue> {
        let index = self.resolve_valid_index(index, false)?;
        Ok(&self.items[index])
    }

    pub(super) fn resolve_valid_index(
        &self,
        index: &ExpressionValue,
        is_exclusive: bool,
    ) -> ExecutionResult<usize> {
        match index {
            ExpressionValue::Integer(int) => {
                self.resolve_valid_index_from_integer(int, is_exclusive)
            }
            _ => index.execution_err("The index must be an integer"),
        }
    }

    fn resolve_valid_index_from_integer(
        &self,
        integer: &ExpressionInteger,
        is_exclusive: bool,
    ) -> ExecutionResult<usize> {
        let span_range = integer.span_range;
        let index = integer.expect_usize()?;
        if is_exclusive {
            if index <= self.items.len() {
                Ok(index)
            } else {
                span_range.execution_err(format!(
                    "Exclusive index of {} must be less than or equal to the array length of {}",
                    index,
                    self.items.len()
                ))
            }
        } else if index < self.items.len() {
            Ok(index)
        } else {
            span_range.execution_err(format!(
                "Inclusive index of {} must be less than the array length of {}",
                index,
                self.items.len()
            ))
        }
    }

    pub(crate) fn concat_recursive_into(
        self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
    ) -> ExecutionResult<()> {
        if behaviour.output_array_structure {
            output.push('[');
        }
        let mut is_first = true;
        for item in self.items {
            if !is_first && behaviour.output_array_structure {
                output.push(',');
            }
            if !is_first && behaviour.add_space_between_token_trees {
                output.push(' ');
            }
            item.concat_recursive_into(output, behaviour)?;
            is_first = false;
        }
        if behaviour.output_array_structure {
            output.push(']');
        }
        Ok(())
    }
}

impl HasSpanRange for ExpressionArray {
    fn span_range(&self) -> SpanRange {
        self.span_range
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
