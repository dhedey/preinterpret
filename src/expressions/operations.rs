use super::*;

pub(super) trait Operation: HasSpanRange {
    fn output(&self, output_value: impl ToEvaluationValue) -> ExecutionResult<EvaluationValue> {
        Ok(output_value.to_value(self.source_span_for_output()))
    }

    fn output_if_some(
        &self,
        output_value: Option<impl ToEvaluationValue>,
        error_message: impl FnOnce() -> String,
    ) -> ExecutionResult<EvaluationValue> {
        match output_value {
            Some(output_value) => self.output(output_value),
            None => self.execution_err(error_message()),
        }
    }

    fn unsupported_for_value_type_err(
        &self,
        value_type: &'static str,
    ) -> ExecutionResult<EvaluationValue> {
        Err(self.execution_error(format!(
            "The {} operator is not supported for {} values",
            self.symbol(),
            value_type,
        )))
    }

    fn source_span_for_output(&self) -> Option<Span>;
    fn symbol(&self) -> &'static str;
}

#[derive(Clone)]
pub(super) enum UnaryOperation {
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

impl UnaryOperation {
    pub(super) fn parse_from_prefix_punct(input: ParseStream) -> ParseResult<Self> {
        if input.peek(Token![-]) {
            Ok(Self::Neg {
                token: input.parse()?,
            })
        } else if input.peek(Token![!]) {
            Ok(Self::Not {
                token: input.parse()?,
            })
        } else {
            input.parse_err("Expected ! or -")
        }
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
        Ok(Self::Cast { as_token, target })
    }

    pub(super) fn evaluate(self, input: EvaluationValue) -> ExecutionResult<EvaluationValue> {
        input.handle_unary_operation(self)
    }
}

impl Operation for UnaryOperation {
    fn source_span_for_output(&self) -> Option<Span> {
        match self {
            UnaryOperation::Neg { .. } => None,
            UnaryOperation::Not { .. } => None,
            UnaryOperation::GroupedNoOp { span } => Some(*span),
            UnaryOperation::Cast { .. } => None,
        }
    }

    fn symbol(&self) -> &'static str {
        match self {
            UnaryOperation::Neg { .. } => "-",
            UnaryOperation::Not { .. } => "!",
            UnaryOperation::GroupedNoOp { .. } => "",
            UnaryOperation::Cast { .. } => "as",
        }
    }
}

impl HasSpan for UnaryOperation {
    fn span(&self) -> Span {
        match self {
            UnaryOperation::Neg { token } => token.span,
            UnaryOperation::Not { token } => token.span,
            UnaryOperation::GroupedNoOp { span } => *span,
            UnaryOperation::Cast { as_token, .. } => as_token.span,
        }
    }
}

pub(super) trait HandleUnaryOperation: Sized {
    fn handle_unary_operation(self, operation: &UnaryOperation)
        -> ExecutionResult<EvaluationValue>;
}

#[derive(Clone)]
pub(super) enum BinaryOperation {
    Paired(PairedBinaryOperation),
    Integer(IntegerBinaryOperation),
}

impl Parse for BinaryOperation {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        // In line with Syn's BinOp, we use peek instead of lookahead
        // ...I assume for slightly increased performance
        // ...Or becuase 30 alternative options in the error message is too many
        if input.peek(Token![+]) {
            Ok(Self::Paired(PairedBinaryOperation::Addition(
                input.parse()?,
            )))
        } else if input.peek(Token![-]) {
            Ok(Self::Paired(PairedBinaryOperation::Subtraction(
                input.parse()?,
            )))
        } else if input.peek(Token![*]) {
            Ok(Self::Paired(PairedBinaryOperation::Multiplication(
                input.parse()?,
            )))
        } else if input.peek(Token![/]) {
            Ok(Self::Paired(PairedBinaryOperation::Division(
                input.parse()?,
            )))
        } else if input.peek(Token![%]) {
            Ok(Self::Paired(PairedBinaryOperation::Remainder(
                input.parse()?,
            )))
        } else if input.peek(Token![&&]) {
            Ok(Self::Paired(PairedBinaryOperation::LogicalAnd(
                input.parse()?,
            )))
        } else if input.peek(Token![||]) {
            Ok(Self::Paired(PairedBinaryOperation::LogicalOr(
                input.parse()?,
            )))
        } else if input.peek(Token![==]) {
            Ok(Self::Paired(PairedBinaryOperation::Equal(input.parse()?)))
        } else if input.peek(Token![!=]) {
            Ok(Self::Paired(PairedBinaryOperation::NotEqual(
                input.parse()?,
            )))
        } else if input.peek(Token![>=]) {
            Ok(Self::Paired(PairedBinaryOperation::GreaterThanOrEqual(
                input.parse()?,
            )))
        } else if input.peek(Token![<=]) {
            Ok(Self::Paired(PairedBinaryOperation::LessThanOrEqual(
                input.parse()?,
            )))
        } else if input.peek(Token![<<]) {
            Ok(Self::Integer(IntegerBinaryOperation::ShiftLeft(
                input.parse()?,
            )))
        } else if input.peek(Token![>>]) {
            Ok(Self::Integer(IntegerBinaryOperation::ShiftRight(
                input.parse()?,
            )))
        } else if input.peek(Token![>]) {
            Ok(Self::Paired(PairedBinaryOperation::GreaterThan(
                input.parse()?,
            )))
        } else if input.peek(Token![<]) {
            Ok(Self::Paired(PairedBinaryOperation::LessThan(
                input.parse()?,
            )))
        } else if input.peek(Token![&]) {
            Ok(Self::Paired(PairedBinaryOperation::BitAnd(input.parse()?)))
        } else if input.peek(Token![|]) {
            Ok(Self::Paired(PairedBinaryOperation::BitOr(input.parse()?)))
        } else if input.peek(Token![^]) {
            Ok(Self::Paired(PairedBinaryOperation::BitXor(input.parse()?)))
        } else {
            input.parse_err("Expected one of + - * / % && || ^ & | == < <= != >= > << or >>")
        }
    }
}

impl BinaryOperation {
    pub(super) fn lazy_evaluate(
        &self,
        left: &EvaluationValue,
    ) -> ExecutionResult<Option<EvaluationValue>> {
        match self {
            BinaryOperation::Paired(PairedBinaryOperation::LogicalAnd { .. }) => {
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
            BinaryOperation::Paired(PairedBinaryOperation::LogicalOr { .. }) => {
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
        &self,
        left: EvaluationValue,
        right: EvaluationValue,
    ) -> ExecutionResult<EvaluationValue> {
        match self {
            BinaryOperation::Paired(operation) => {
                let value_pair = left.expect_value_pair(operation, right)?;
                value_pair.handle_paired_binary_operation(operation)
            }
            BinaryOperation::Integer(operation) => {
                let right = right
                    .into_integer()
                    .ok_or_else(|| self.execution_error("The shift amount must be an integer"))?;
                left.handle_integer_binary_operation(right, operation)
            }
        }
    }
}

impl HasSpan for BinaryOperation {
    fn span(&self) -> Span {
        match self {
            BinaryOperation::Paired(_) => self.span(),
            BinaryOperation::Integer(_) => self.span(),
        }
    }
}

impl Operation for BinaryOperation {
    fn source_span_for_output(&self) -> Option<Span> {
        None
    }

    fn symbol(&self) -> &'static str {
        match self {
            BinaryOperation::Paired(paired) => paired.symbol(),
            BinaryOperation::Integer(integer) => integer.symbol(),
        }
    }
}

#[derive(Copy, Clone)]
pub(super) enum PairedBinaryOperation {
    Addition(Token![+]),
    Subtraction(Token![-]),
    Multiplication(Token![*]),
    Division(Token![/]),
    Remainder(Token![%]),
    LogicalAnd(Token![&&]),
    LogicalOr(Token![||]),
    BitXor(Token![^]),
    BitAnd(Token![&]),
    BitOr(Token![|]),
    Equal(Token![==]),
    LessThan(Token![<]),
    LessThanOrEqual(Token![<=]),
    NotEqual(Token![!=]),
    GreaterThanOrEqual(Token![>=]),
    GreaterThan(Token![>]),
}

impl Operation for PairedBinaryOperation {
    fn source_span_for_output(&self) -> Option<Span> {
        None
    }

    fn symbol(&self) -> &'static str {
        match self {
            PairedBinaryOperation::Addition { .. } => "+",
            PairedBinaryOperation::Subtraction { .. } => "-",
            PairedBinaryOperation::Multiplication { .. } => "*",
            PairedBinaryOperation::Division { .. } => "/",
            PairedBinaryOperation::Remainder { .. } => "%",
            PairedBinaryOperation::LogicalAnd { .. } => "&&",
            PairedBinaryOperation::LogicalOr { .. } => "||",
            PairedBinaryOperation::BitXor { .. } => "^",
            PairedBinaryOperation::BitAnd { .. } => "&",
            PairedBinaryOperation::BitOr { .. } => "|",
            PairedBinaryOperation::Equal { .. } => "==",
            PairedBinaryOperation::LessThan { .. } => "<",
            PairedBinaryOperation::LessThanOrEqual { .. } => "<=",
            PairedBinaryOperation::NotEqual { .. } => "!=",
            PairedBinaryOperation::GreaterThanOrEqual { .. } => ">=",
            PairedBinaryOperation::GreaterThan { .. } => ">",
        }
    }
}

impl HasSpanRange for PairedBinaryOperation {
    fn span_range(&self) -> SpanRange {
        match self {
            PairedBinaryOperation::Addition(plus) => plus.span_range(),
            PairedBinaryOperation::Subtraction(minus) => minus.span_range(),
            PairedBinaryOperation::Multiplication(star) => star.span_range(),
            PairedBinaryOperation::Division(slash) => slash.span_range(),
            PairedBinaryOperation::Remainder(percent) => percent.span_range(),
            PairedBinaryOperation::LogicalAnd(and_and) => and_and.span_range(),
            PairedBinaryOperation::LogicalOr(or_or) => or_or.span_range(),
            PairedBinaryOperation::BitXor(caret) => caret.span_range(),
            PairedBinaryOperation::BitAnd(and) => and.span_range(),
            PairedBinaryOperation::BitOr(or) => or.span_range(),
            PairedBinaryOperation::Equal(eq_eq) => eq_eq.span_range(),
            PairedBinaryOperation::LessThan(lt) => lt.span_range(),
            PairedBinaryOperation::LessThanOrEqual(le) => le.span_range(),
            PairedBinaryOperation::NotEqual(ne) => ne.span_range(),
            PairedBinaryOperation::GreaterThanOrEqual(ge) => ge.span_range(),
            PairedBinaryOperation::GreaterThan(gt) => gt.span_range(),
        }
    }
}

#[derive(Copy, Clone)]
pub(super) enum IntegerBinaryOperation {
    ShiftLeft(Token![<<]),
    ShiftRight(Token![>>]),
}

impl Operation for IntegerBinaryOperation {
    fn source_span_for_output(&self) -> Option<Span> {
        None
    }

    fn symbol(&self) -> &'static str {
        match self {
            IntegerBinaryOperation::ShiftLeft { .. } => "<<",
            IntegerBinaryOperation::ShiftRight { .. } => ">>",
        }
    }
}

impl HasSpanRange for IntegerBinaryOperation {
    fn span_range(&self) -> SpanRange {
        match self {
            IntegerBinaryOperation::ShiftLeft(shl) => shl.span_range(),
            IntegerBinaryOperation::ShiftRight(shr) => shr.span_range(),
        }
    }
}

pub(super) trait HandleBinaryOperation: Sized {
    fn handle_paired_binary_operation(
        self,
        rhs: Self,
        operation: &PairedBinaryOperation,
    ) -> ExecutionResult<EvaluationValue>;

    fn handle_integer_binary_operation(
        self,
        rhs: EvaluationInteger,
        operation: &IntegerBinaryOperation,
    ) -> ExecutionResult<EvaluationValue>;
}
