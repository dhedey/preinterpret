use super::*;

#[derive(Clone)]
pub(crate) struct ExpressionStream {
    pub(crate) value: OutputStream,
    /// The span range that generated this value.
    /// For a complex expression, the start span is the most left part
    /// of the expression, and the end span is the most right part.
    pub(crate) span_range: SpanRange,
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
                CastTarget::String => operation.output({
                    let mut output = String::new();
                    self.concat_recursive_into(&mut output, &ConcatBehaviour::standard());
                    output
                }),
                CastTarget::Stream => operation.output(self.value),
                CastTarget::Group => {
                    operation.output(operation.output(self.value).into_new_output_stream(
                        Grouping::Grouped,
                        StreamOutputBehaviour::Standard,
                    )?)
                }
                CastTarget::Boolean
                | CastTarget::Char
                | CastTarget::Integer(_)
                | CastTarget::Float(_) => {
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

    pub(crate) fn concat_recursive_into(self, output: &mut String, behaviour: &ConcatBehaviour) {
        if behaviour.output_types_as_commands {
            if self.value.is_empty() {
                output.push_str("[!stream!]");
            } else {
                output.push_str("[!stream! ");
                self.value.concat_recursive_into(output, behaviour);
                output.push(']');
            }
        } else {
            self.value.concat_recursive_into(output, behaviour);
        }
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

impl ToExpressionValue for TokenStream {
    fn to_value(self, span_range: SpanRange) -> ExpressionValue {
        OutputStream::raw(self).to_value(span_range)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct StreamTypeData;

impl MethodResolutionTarget for StreamTypeData {
    type Parent = ValueTypeData;
    const PARENT: Option<Self::Parent> = Some(ValueTypeData);

    fn resolve_own_method(method_name: &str) -> Option<MethodInterface> {
        define_method_matcher! {
            (match method_name on Self)

            fn len(this: Shared<ExpressionStream>) -> ExecutionResult<usize> {
                Ok(this.value.len())
            }

            fn flatten(this: Owned<ExpressionStream>) -> ExecutionResult<TokenStream> {
                Ok(this.into_inner().value.into_token_stream_removing_any_transparent_groups())
            }

            fn infer(this: Owned<ExpressionStream>) -> ExecutionResult<ExpressionValue> {
                let span_range = this.span_range();
                Ok(this.into_inner().value.coerce_into_value(span_range))
            }
        }
    }
}
