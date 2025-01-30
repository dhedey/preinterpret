use super::*;

#[derive(Clone)]
pub(super) struct UnaryOperation {
    pub(super) operator: UnaryOperator,
}

impl UnaryOperation {
    fn error(&self, error_message: &str) -> ExecutionInterrupt {
        self.operator.execution_error(error_message)
    }

    pub(super) fn unsupported_for_value_type_err(
        &self,
        value_type: &'static str,
    ) -> ExecutionResult<EvaluationValue> {
        Err(self.error(&format!(
            "The {} operator is not supported for {} values",
            self.operator.symbol(),
            value_type,
        )))
    }

    pub(super) fn err(&self, error_message: &'static str) -> ExecutionResult<EvaluationValue> {
        Err(self.error(error_message))
    }

    pub(super) fn output(
        &self,
        output_value: impl ToEvaluationValue,
    ) -> ExecutionResult<EvaluationValue> {
        Ok(output_value.to_value(self.operator.source_span_for_output()))
    }

    pub(super) fn for_cast_operation(
        as_token: Token![as],
        target_type: Ident,
    ) -> ParseResult<Self> {
        let target = match target_type.to_string().as_str() {
            "int" | "integer" => CastTarget::Integer(IntegerKind::Untyped),
            "u8" => CastTarget::Integer(IntegerKind::U8),
            "u16" => CastTarget::Integer(IntegerKind::U16),
            "u32" => CastTarget::Integer(IntegerKind::U32),
            "u64" => CastTarget::Integer(IntegerKind::U64),
            "u128" => CastTarget::Integer(IntegerKind::U128),
            "usize" => CastTarget::Integer(IntegerKind::Usize),
            "i8" => CastTarget::Integer(IntegerKind::I8),
            "i16" => CastTarget::Integer(IntegerKind::I16),
            "i32" => CastTarget::Integer(IntegerKind::I32),
            "i64" => CastTarget::Integer(IntegerKind::I64),
            "i128" => CastTarget::Integer(IntegerKind::I128),
            "isize" => CastTarget::Integer(IntegerKind::Isize),
            "float" => CastTarget::Float(FloatKind::Untyped),
            "f32" => CastTarget::Float(FloatKind::F32),
            "f64" => CastTarget::Float(FloatKind::F64),
            "bool" => CastTarget::Boolean,
            "char" => CastTarget::Char,
            _ => {
                return target_type
                    .parse_err("This type is not supported in preinterpret cast expressions")
            }
        };
        Ok(Self {
            operator: UnaryOperator::Cast { as_token, target },
        })
    }

    pub(super) fn for_unary_operator(operator: syn::UnOp) -> ParseResult<Self> {
        let operator = match operator {
            UnOp::Neg(token) => UnaryOperator::Neg { token },
            UnOp::Not(token) => UnaryOperator::Not { token },
            other_unary_op => {
                return other_unary_op.parse_err(
                    "This unary operator is not supported in a preinterpret expression",
                );
            }
        };
        Ok(Self { operator })
    }

    pub(super) fn evaluate(self, input: EvaluationValue) -> ExecutionResult<EvaluationValue> {
        input.handle_unary_operation(self)
    }
}

#[derive(Copy, Clone)]
pub(super) enum UnaryOperator {
    Neg {
        token: Token![-],
    },
    Not {
        token: Token![!],
    },
    GroupedNoOp {
        span: Span,
    },
    Cast {
        as_token: Token![as],
        target: CastTarget,
    },
}

impl UnaryOperator {
    pub(crate) fn source_span_for_output(&self) -> Option<Span> {
        match self {
            UnaryOperator::Neg { .. } => None,
            UnaryOperator::Not { .. } => None,
            UnaryOperator::GroupedNoOp { span } => Some(*span),
            UnaryOperator::Cast { .. } => None,
        }
    }

    pub(crate) fn symbol(&self) -> &'static str {
        match self {
            UnaryOperator::Neg { .. } => "-",
            UnaryOperator::Not { .. } => "!",
            UnaryOperator::GroupedNoOp { .. } => "",
            UnaryOperator::Cast { .. } => "as",
        }
    }
}

impl HasSpan for UnaryOperator {
    fn span(&self) -> Span {
        match self {
            UnaryOperator::Neg { token } => token.span,
            UnaryOperator::Not { token } => token.span,
            UnaryOperator::GroupedNoOp { span } => *span,
            UnaryOperator::Cast { as_token, .. } => as_token.span,
        }
    }
}

pub(super) trait HandleUnaryOperation: Sized {
    fn handle_unary_operation(self, operation: &UnaryOperation)
        -> ExecutionResult<EvaluationValue>;
}

#[derive(Clone)]
pub(super) struct BinaryOperation {
    /// Only present if there is a single span for the source tokens
    pub(super) source_span: Option<Span>,
    pub(super) operator_span: Span,
    pub(super) operator: BinaryOperator,
}

impl BinaryOperation {
    fn error(&self, error_message: &str) -> ExecutionInterrupt {
        self.operator_span.execution_error(error_message)
    }

    pub(super) fn unsupported_for_value_type_err(
        &self,
        value_type: &'static str,
    ) -> ExecutionResult<EvaluationValue> {
        Err(self.error(&format!(
            "The {} operator is not supported for {} values",
            self.operator.symbol(),
            value_type,
        )))
    }

    pub(super) fn output(
        &self,
        output_value: impl ToEvaluationValue,
    ) -> ExecutionResult<EvaluationValue> {
        Ok(output_value.to_value(self.source_span))
    }

    pub(super) fn output_if_some(
        &self,
        output_value: Option<impl ToEvaluationValue>,
        error_message: impl FnOnce() -> String,
    ) -> ExecutionResult<EvaluationValue> {
        match output_value {
            Some(output_value) => self.output(output_value),
            None => self.operator_span.execution_err(error_message()),
        }
    }

    pub(super) fn for_binary_operator(syn_operator: syn::BinOp) -> ParseResult<Self> {
        let operator = match syn_operator {
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
                    .parse_err("This operation is not supported in preinterpret expressions")
            }
        };
        Ok(Self {
            source_span: None,
            operator_span: syn_operator.span_range().join_into_span_else_start(),
            operator,
        })
    }

    pub(super) fn lazy_evaluate(
        &self,
        left: &EvaluationValue,
    ) -> ExecutionResult<Option<EvaluationValue>> {
        match self.operator {
            BinaryOperator::Paired(PairedBinaryOperator::LogicalAnd) => {
                match left.clone().into_bool() {
                    Some(bool) => {
                        if !bool.value {
                            Ok(Some(EvaluationValue::Boolean(bool)))
                        } else {
                            Ok(None)
                        }
                    }
                    None => self.execution_err("The left operand was not a boolean"),
                }
            }
            BinaryOperator::Paired(PairedBinaryOperator::LogicalOr) => {
                match left.clone().into_bool() {
                    Some(bool) => {
                        if bool.value {
                            Ok(Some(EvaluationValue::Boolean(bool)))
                        } else {
                            Ok(None)
                        }
                    }
                    None => self.execution_err("The left operand was not a boolean"),
                }
            }
            _ => Ok(None),
        }
    }

    pub(super) fn evaluate(
        self,
        left: EvaluationValue,
        right: EvaluationValue,
    ) -> ExecutionResult<EvaluationValue> {
        match self.operator {
            BinaryOperator::Paired(operator) => {
                let value_pair = left.expect_value_pair(operator, right, self.operator_span)?;
                value_pair.handle_paired_binary_operation(self)
            }
            BinaryOperator::Integer(_) => {
                let right = right.into_integer().ok_or_else(|| {
                    self.operator_span
                        .execution_error("The shift amount must be an integer")
                })?;
                left.handle_integer_binary_operation(right, self)
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

impl HasSpan for BinaryOperation {
    fn span(&self) -> Span {
        self.operator_span
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
    ) -> ExecutionResult<EvaluationValue>;

    fn handle_integer_binary_operation(
        self,
        rhs: EvaluationInteger,
        operation: &BinaryOperation,
    ) -> ExecutionResult<EvaluationValue>;
}
