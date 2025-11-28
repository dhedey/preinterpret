use super::*;

#[derive(Clone)]
pub(crate) struct ArrayExpression {
    pub(crate) items: Vec<ExpressionValue>,
}

impl ArrayExpression {
    pub(crate) fn new(items: Vec<ExpressionValue>) -> Self {
        Self { items }
    }

    pub(crate) fn output_items_to(
        &self,
        output: &mut ToStreamContext,
        grouping: Grouping,
    ) -> ExecutionResult<()> {
        for item in &self.items {
            item.output_to(grouping, output)?;
        }
        Ok(())
    }

    pub(super) fn handle_integer_binary_operation(
        self,
        _right: IntegerExpression,
        operation: WrappedOp<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue> {
        operation.unsupported(self)
    }

    pub(super) fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: WrappedOp<PairedBinaryOperation>,
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
        index: Spanned<&ExpressionValue>,
    ) -> ExecutionResult<ExpressionValue> {
        let (index, span_range) = index.deconstruct();
        Ok(match index {
            ExpressionValue::Integer(integer) => {
                let index =
                    self.resolve_valid_index_from_integer(integer.spanned(span_range), false)?;
                std::mem::replace(&mut self.items[index], ExpressionValue::None)
            }
            ExpressionValue::Range(range) => {
                let range = range.spanned(span_range).resolve_to_index_range(&self)?;
                let new_items: Vec<_> = self.items.drain(range).collect();
                new_items.into_value()
            }
            _ => return span_range.type_err("The index must be an integer or a range"),
        })
    }

    pub(super) fn index_mut(
        &mut self,
        index: Spanned<&ExpressionValue>,
    ) -> ExecutionResult<&mut ExpressionValue> {
        let (index, span_range) = index.deconstruct();
        Ok(match index {
            ExpressionValue::Integer(integer) => {
                let index =
                    self.resolve_valid_index_from_integer(integer.spanned(span_range), false)?;
                &mut self.items[index]
            }
            ExpressionValue::Range(..) => {
                // Temporary until we add slice types - we error here
                return span_range.ownership_err("Currently, a range-indexed array must be owned. Use `.take()` or `.clone()` before indexing [..]");
            }
            _ => return span_range.type_err("The index must be an integer or a range"),
        })
    }

    pub(super) fn index_ref(
        &self,
        index: Spanned<&ExpressionValue>,
    ) -> ExecutionResult<&ExpressionValue> {
        let (index, span_range) = index.deconstruct();
        Ok(match index {
            ExpressionValue::Integer(integer) => {
                let index =
                    self.resolve_valid_index_from_integer(integer.spanned(span_range), false)?;
                &self.items[index]
            }
            ExpressionValue::Range(..) => {
                // Temporary until we add slice types - we error here
                return span_range.ownership_err("Currently, a range-indexed array must be owned. Use `.take()` or `.clone()` before indexing [..]");
            }
            _ => return span_range.type_err("The index must be an integer or a range"),
        })
    }

    pub(super) fn resolve_valid_index(
        &self,
        index: Spanned<&ExpressionValue>,
        is_exclusive: bool,
    ) -> ExecutionResult<usize> {
        let (index, span_range) = index.deconstruct();
        match index {
            ExpressionValue::Integer(int) => {
                self.resolve_valid_index_from_integer(int.spanned(span_range), is_exclusive)
            }
            _ => span_range.type_err("The index must be an integer"),
        }
    }

    fn resolve_valid_index_from_integer(
        &self,
        integer: Spanned<&IntegerExpression>,
        is_exclusive: bool,
    ) -> ExecutionResult<usize> {
        let index: usize = integer
            .clone()
            .into_owned_value(integer.span_range)
            .resolve_as("An array index")?;
        if is_exclusive {
            if index <= self.items.len() {
                Ok(index)
            } else {
                integer.value_err(format!(
                    "Exclusive index of {} must be less than or equal to the array length of {}",
                    index,
                    self.items.len()
                ))
            }
        } else if index < self.items.len() {
            Ok(index)
        } else {
            integer.value_err(format!(
                "Inclusive index of {} must be less than the array length of {}",
                index,
                self.items.len()
            ))
        }
    }

    pub(crate) fn concat_recursive_into(
        &self,
        output: &mut String,
        behaviour: &ConcatBehaviour,
    ) -> ExecutionResult<()> {
        IteratorExpression::any_iterator_to_string(
            self.items.iter(),
            output,
            behaviour,
            "[]",
            "[",
            "]",
            false, // Output all the vec because it's already in memory
        )
    }
}

impl HasValueType for ArrayExpression {
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
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Array(ArrayExpression { items: self })
    }
}

impl ToExpressionValue for ArrayExpression {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Array(self)
    }
}

define_interface! {
    struct ArrayTypeData,
    parent: IterableTypeData,
    pub(crate) mod array_interface {
        pub(crate) mod methods {
            fn push(mut this: Mutable<ArrayExpression>, item: OwnedValue) -> ExecutionResult<()> {
                this.items.push(item.into());
                Ok(())
            }

            [context] fn to_stream_grouped(this: ArrayExpression) -> StreamOutput<impl StreamAppender> {
                let error_span_range = context.span_range();
                StreamOutput::new(move |stream| this.output_items_to(&mut ToStreamContext::new(stream, error_span_range), Grouping::Grouped))
            }
        }
        pub(crate) mod unary_operations {
            [context] fn cast_to_numeric(this: Owned<ArrayExpression>) -> ExecutionResult<ResolvedValue> {
                let (mut this, span_range) = this.deconstruct();
                let length = this.items.len();
                if length == 1 {
                    context.operation.evaluate(this.items.pop().unwrap().into_owned(span_range))
                } else {
                    context.operation.value_err(format!(
                        "Only a singleton array can be cast to this value but the array has {} elements",
                        length,
                    ))
                }
            }
        }
        interface_items {
            fn resolve_own_unary_operation(operation: &UnaryOperation) -> Option<UnaryOperationInterface> {
                Some(match operation {
                    UnaryOperation::Neg { .. } | UnaryOperation::Not { .. } => return None,
                    UnaryOperation::Cast { target, .. } => match target {
                        CastTarget::Boolean
                        | CastTarget::Char
                        | CastTarget::Integer(_)
                        | CastTarget::Float(_) => unary_definitions::cast_to_numeric(),
                        _ => return None,
                    },
                })
            }
        }
    }
}
