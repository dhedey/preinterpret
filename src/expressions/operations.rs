use super::*;

pub(super) enum EvaluationOperator {
    Unary {
        operation: UnaryOperation,
        input: Option<EvaluationOutput>,
    },
    Binary {
        operation: BinaryOperation,
        left_input: Option<EvaluationOutput>,
        right_input: Option<EvaluationOutput>,
    },
}

impl EvaluationOperator {
    pub(super) fn evaluate(self) -> Result<EvaluationOutput> {
        const OPERATOR_INPUT_EXPECT_STR: &str = "Handling children on the stack ordering should ensure the parent input is always set when the parent is evaluated";

        match self {
            Self::Unary {
                operation: operator,
                input,
            } => operator.evaluate(input.expect(OPERATOR_INPUT_EXPECT_STR)),
            Self::Binary {
                operation: operator,
                left_input,
                right_input,
            } => operator.evaluate(
                left_input.expect(OPERATOR_INPUT_EXPECT_STR),
                right_input.expect(OPERATOR_INPUT_EXPECT_STR),
            ),
        }
    }
}

pub(super) struct UnaryOperation {
    pub(super) span_for_output: SpanRange,
    pub(super) operator_span: SpanRange,
    pub(super) operator: UnaryOperator,
}

impl UnaryOperation {
    fn error(&self, error_message: &str) -> syn::Error {
        self.operator_span.error(error_message)
    }

    pub(super) fn unsupported_for_value_type_err(
        &self,
        value_type: &'static str,
    ) -> Result<EvaluationOutput> {
        Err(self.error(&format!(
            "The {} operator is not supported for {} values",
            self.operator.symbol(),
            value_type,
        )))
    }

    pub(super) fn err(&self, error_message: &'static str) -> Result<EvaluationOutput> {
        Err(self.error(error_message))
    }

    pub(super) fn output(&self, output_value: impl ToEvaluationOutput) -> Result<EvaluationOutput> {
        Ok(output_value.to_output(self.span_for_output))
    }

    pub(super) fn for_cast_expression(expr: &syn::ExprCast) -> Result<Self> {
        fn extract_type(ty: &syn::Type) -> Result<ValueKind> {
            match ty {
                syn::Type::Group(group) => extract_type(&group.elem),
                syn::Type::Path(type_path)
                    if type_path.qself.is_none()
                        && type_path.path.leading_colon.is_none()
                        && type_path.path.segments.len() == 1 =>
                {
                    let Some(ident) = type_path.path.get_ident() else {
                        return type_path
                            .span_range()
                            .err("This type is not supported in preinterpret cast expressions");
                    };
                    match ident.to_string().as_str() {
                        "int" | "integer" => Ok(ValueKind::Integer(IntegerKind::Untyped)),
                        "u8" => Ok(ValueKind::Integer(IntegerKind::U8)),
                        "u16" => Ok(ValueKind::Integer(IntegerKind::U16)),
                        "u32" => Ok(ValueKind::Integer(IntegerKind::U32)),
                        "u64" => Ok(ValueKind::Integer(IntegerKind::U64)),
                        "u128" => Ok(ValueKind::Integer(IntegerKind::U128)),
                        "usize" => Ok(ValueKind::Integer(IntegerKind::Usize)),
                        "i8" => Ok(ValueKind::Integer(IntegerKind::I8)),
                        "i16" => Ok(ValueKind::Integer(IntegerKind::I16)),
                        "i32" => Ok(ValueKind::Integer(IntegerKind::I32)),
                        "i64" => Ok(ValueKind::Integer(IntegerKind::I64)),
                        "i128" => Ok(ValueKind::Integer(IntegerKind::I128)),
                        "isize" => Ok(ValueKind::Integer(IntegerKind::Isize)),
                        "float" => Ok(ValueKind::Float(FloatKind::Untyped)),
                        "f32" => Ok(ValueKind::Float(FloatKind::F32)),
                        "f64" => Ok(ValueKind::Float(FloatKind::F64)),
                        "bool" => Ok(ValueKind::Boolean),
                        _ => ident
                            .span()
                            .span_range()
                            .err("This type is not supported in preinterpret cast expressions"),
                    }
                }
                other => other
                    .span_range()
                    .err("This type is not supported in preinterpret cast expressions"),
            }
        }

        Ok(Self {
            span_for_output: expr.expr.span_range(),
            operator_span: expr.as_token.span.span_range(),
            operator: UnaryOperator::Cast(extract_type(&expr.ty)?),
        })
    }

    pub(super) fn for_group_expression(expr: &syn::ExprGroup) -> Result<Self> {
        Ok(Self {
            span_for_output: expr.group_token.span.span_range(),
            operator_span: expr.group_token.span.span_range(),
            operator: UnaryOperator::NoOp,
        })
    }

    pub(super) fn for_paren_expression(expr: &syn::ExprParen) -> Result<Self> {
        Ok(Self {
            span_for_output: expr.paren_token.span.span_range(),
            operator_span: expr.paren_token.span.span_range(),
            operator: UnaryOperator::NoOp,
        })
    }

    pub(super) fn for_unary_expression(expr: &syn::ExprUnary) -> Result<Self> {
        let operator = match &expr.op {
            UnOp::Neg(_) => UnaryOperator::Neg,
            UnOp::Not(_) => UnaryOperator::Not,
            other_unary_op => {
                return other_unary_op
                    .span_range()
                    .err("This unary operator is not supported in preinterpret expressions");
            }
        };
        Ok(Self {
            span_for_output: expr.span_range(),
            operator_span: expr.op.span_range(),
            operator,
        })
    }

    pub(super) fn evaluate(self, input: EvaluationOutput) -> Result<EvaluationOutput> {
        input.into_value().handle_unary_operation(self)
    }
}

#[derive(Copy, Clone)]
pub(super) enum UnaryOperator {
    Neg,
    Not,
    NoOp,
    Cast(ValueKind),
}

impl UnaryOperator {
    pub(crate) fn symbol(&self) -> &'static str {
        match self {
            UnaryOperator::Neg => "-",
            UnaryOperator::Not => "!",
            UnaryOperator::NoOp => "",
            UnaryOperator::Cast(_) => "as",
        }
    }
}

pub(super) trait HandleUnaryOperation: Sized {
    fn handle_unary_operation(self, operation: &UnaryOperation) -> Result<EvaluationOutput>;
}

pub(super) struct BinaryOperation {
    pub(super) span_for_output: SpanRange,
    pub(super) operator_span: SpanRange,
    pub(super) operator: BinaryOperator,
}

impl BinaryOperation {
    fn error(&self, error_message: &str) -> syn::Error {
        self.operator_span.error(error_message)
    }

    pub(super) fn unsupported_for_value_type_err(
        &self,
        value_type: &'static str,
    ) -> Result<EvaluationOutput> {
        Err(self.error(&format!(
            "The {} operator is not supported for {} values",
            self.operator.symbol(),
            value_type,
        )))
    }

    pub(super) fn output(&self, output_value: impl ToEvaluationOutput) -> Result<EvaluationOutput> {
        Ok(output_value.to_output(self.span_for_output))
    }

    pub(super) fn output_if_some(
        &self,
        output_value: Option<impl ToEvaluationOutput>,
        error_message: impl FnOnce() -> String,
    ) -> Result<EvaluationOutput> {
        match output_value {
            Some(output_value) => self.output(output_value),
            None => Err(self.operator_span.error(error_message())),
        }
    }

    pub(super) fn for_binary_expression(expr: &syn::ExprBinary) -> Result<Self> {
        let operator = match &expr.op {
            syn::BinOp::Add(_) => BinaryOperator::Paired(PairedBinaryOperator::Addition),
            syn::BinOp::Sub(_) => BinaryOperator::Paired(PairedBinaryOperator::Subtraction),
            syn::BinOp::Mul(_) => BinaryOperator::Paired(PairedBinaryOperator::Multiplication),
            syn::BinOp::Div(_) => BinaryOperator::Paired(PairedBinaryOperator::Division),
            syn::BinOp::Rem(_) => BinaryOperator::Paired(PairedBinaryOperator::Remainder),
            syn::BinOp::And(_) => BinaryOperator::Paired(PairedBinaryOperator::LogicalAnd),
            syn::BinOp::Or(_) => BinaryOperator::Paired(PairedBinaryOperator::LogicalOr),
            syn::BinOp::BitXor(_) => BinaryOperator::Paired(PairedBinaryOperator::BitXor),
            syn::BinOp::BitAnd(_) => BinaryOperator::Paired(PairedBinaryOperator::BitAnd),
            syn::BinOp::BitOr(_) => BinaryOperator::Paired(PairedBinaryOperator::BitOr),
            syn::BinOp::Shl(_) => BinaryOperator::Integer(IntegerBinaryOperator::ShiftLeft),
            syn::BinOp::Shr(_) => BinaryOperator::Integer(IntegerBinaryOperator::ShiftRight),
            syn::BinOp::Eq(_) => BinaryOperator::Paired(PairedBinaryOperator::Equal),
            syn::BinOp::Lt(_) => BinaryOperator::Paired(PairedBinaryOperator::LessThan),
            syn::BinOp::Le(_) => BinaryOperator::Paired(PairedBinaryOperator::LessThanOrEqual),
            syn::BinOp::Ne(_) => BinaryOperator::Paired(PairedBinaryOperator::NotEqual),
            syn::BinOp::Ge(_) => BinaryOperator::Paired(PairedBinaryOperator::GreaterThanOrEqual),
            syn::BinOp::Gt(_) => BinaryOperator::Paired(PairedBinaryOperator::GreaterThan),
            other_binary_operation => {
                return other_binary_operation
                    .span_range()
                    .err("This operation is not supported in preinterpret expressions")
            }
        };
        Ok(Self {
            span_for_output: expr.span_range(),
            operator_span: expr.op.span_range(),
            operator,
        })
    }

    fn evaluate(self, left: EvaluationOutput, right: EvaluationOutput) -> Result<EvaluationOutput> {
        match self.operator {
            BinaryOperator::Paired(operator) => {
                let value_pair = left.expect_value_pair(operator, right, self.operator_span)?;
                value_pair.handle_paired_binary_operation(self)
            }
            BinaryOperator::Integer(_) => {
                let right = right.expect_integer("The shift amount must be an integer")?;
                left.into_value()
                    .handle_integer_binary_operation(right, self)
            }
        }
    }

    pub(super) fn paired_operator(&self) -> PairedBinaryOperator {
        match self.operator {
            BinaryOperator::Paired(operator) => operator,
            BinaryOperator::Integer(_) => panic!("Expected a paired operator"),
        }
    }

    pub(super) fn integer_operator(&self) -> IntegerBinaryOperator {
        match self.operator {
            BinaryOperator::Paired(_) => panic!("Expected an integer operator"),
            BinaryOperator::Integer(operator) => operator,
        }
    }
}

#[derive(Copy, Clone)]
pub(super) enum BinaryOperator {
    Paired(PairedBinaryOperator),
    Integer(IntegerBinaryOperator),
}

impl BinaryOperator {
    pub(super) fn symbol(&self) -> &'static str {
        match self {
            BinaryOperator::Paired(paired) => paired.symbol(),
            BinaryOperator::Integer(integer) => integer.symbol(),
        }
    }
}

#[derive(Copy, Clone)]
pub(super) enum PairedBinaryOperator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Remainder,
    LogicalAnd,
    LogicalOr,
    BitXor,
    BitAnd,
    BitOr,
    Equal,
    LessThan,
    LessThanOrEqual,
    NotEqual,
    GreaterThanOrEqual,
    GreaterThan,
}

impl PairedBinaryOperator {
    pub(super) fn symbol(&self) -> &'static str {
        match self {
            PairedBinaryOperator::Addition => "+",
            PairedBinaryOperator::Subtraction => "-",
            PairedBinaryOperator::Multiplication => "*",
            PairedBinaryOperator::Division => "/",
            PairedBinaryOperator::Remainder => "%",
            PairedBinaryOperator::LogicalAnd => "&&",
            PairedBinaryOperator::LogicalOr => "||",
            PairedBinaryOperator::BitXor => "^",
            PairedBinaryOperator::BitAnd => "&",
            PairedBinaryOperator::BitOr => "|",
            PairedBinaryOperator::Equal => "==",
            PairedBinaryOperator::LessThan => "<",
            PairedBinaryOperator::LessThanOrEqual => "<=",
            PairedBinaryOperator::NotEqual => "!=",
            PairedBinaryOperator::GreaterThanOrEqual => ">=",
            PairedBinaryOperator::GreaterThan => ">",
        }
    }
}

#[derive(Copy, Clone)]
pub(super) enum IntegerBinaryOperator {
    ShiftLeft,
    ShiftRight,
}

impl IntegerBinaryOperator {
    pub(super) fn symbol(&self) -> &'static str {
        match self {
            IntegerBinaryOperator::ShiftLeft => "<<",
            IntegerBinaryOperator::ShiftRight => ">>",
        }
    }
}

pub(super) trait HandleBinaryOperation: Sized {
    fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: &BinaryOperation,
    ) -> Result<EvaluationOutput>;

    fn handle_integer_binary_operation(
        self,
        rhs: EvaluationInteger,
        operation: &BinaryOperation,
    ) -> Result<EvaluationOutput>;
}
