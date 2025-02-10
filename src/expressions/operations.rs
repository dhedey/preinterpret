use super::*;

pub(super) trait Operation: HasSpanRange {
    fn with_output_span_range(&self, output_span_range: SpanRange) -> OutputSpanned<'_, Self> {
        OutputSpanned {
            output_span_range,
            operation: self,
        }
    }

    fn symbol(&self) -> &'static str;
}

pub(super) struct OutputSpanned<'a, T: Operation + ?Sized> {
    pub(super) output_span_range: SpanRange,
    pub(super) operation: &'a T,
}

impl<T: Operation> OutputSpanned<'_, T> {
    pub(super) fn symbol(&self) -> &'static str {
        self.operation.symbol()
    }

    pub(super) fn output(&self, output_value: impl ToExpressionValue) -> ExpressionValue {
        output_value.to_value(self.output_span_range)
    }

    pub(super) fn output_if_some(
        &self,
        output_value: Option<impl ToExpressionValue>,
        error_message: impl FnOnce() -> String,
    ) -> ExecutionResult<ExpressionValue> {
        match output_value {
            Some(output_value) => Ok(self.output(output_value)),
            None => self.operation.execution_err(error_message()),
        }
    }

    pub(super) fn unsupported(&self, value: impl HasValueType) -> ExecutionResult<ExpressionValue> {
        Err(self.operation.execution_error(format!(
            "The {} operator is not supported for {} values",
            self.operation.symbol(),
            value.value_type(),
        )))
    }
}

impl<T: Operation> HasSpanRange for OutputSpanned<'_, T> {
    fn span_range(&self) -> SpanRange {
        // This is used for errors of the operation, so should be targetted
        // to the span range of the _operator_, not the output.
        self.operation.span_range()
    }
}

pub(super) enum PrefixUnaryOperation {
    Neg(Token![-]),
    Not(Token![!]),
}

impl SynParse for PrefixUnaryOperation {
    fn parse(input: SynParseStream) -> SynResult<Self> {
        if input.peek(Token![-]) {
            Ok(Self::Neg(input.parse()?))
        } else if input.peek(Token![!]) {
            Ok(Self::Not(input.parse()?))
        } else {
            Err(input.error("Expected ! or -"))
        }
    }
}

impl HasSpan for PrefixUnaryOperation {
    fn span(&self) -> Span {
        match self {
            PrefixUnaryOperation::Neg(token) => token.span,
            PrefixUnaryOperation::Not(token) => token.span,
        }
    }
}

impl From<PrefixUnaryOperation> for UnaryOperation {
    fn from(operation: PrefixUnaryOperation) -> Self {
        match operation {
            PrefixUnaryOperation::Neg(token) => Self::Neg { token },
            PrefixUnaryOperation::Not(token) => Self::Not { token },
        }
    }
}

#[derive(Clone)]
pub(super) enum UnaryOperation {
    Neg {
        token: Token![-],
    },
    Not {
        token: Token![!],
    },
    Cast {
        as_token: Token![as],
        target_ident: Ident,
        target: CastTarget,
    },
}

impl UnaryOperation {
    pub(super) fn for_cast_operation(
        as_token: Token![as],
        target_ident: Ident,
    ) -> ParseResult<Self> {
        let target = match target_ident.to_string().as_str() {
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
            "stream" => CastTarget::Stream,
            _ => {
                return target_ident
                    .parse_err("This type is not supported in preinterpret cast expressions")
            }
        };
        Ok(Self::Cast {
            as_token,
            target,
            target_ident,
        })
    }

    pub(super) fn evaluate(self, input: ExpressionValue) -> ExecutionResult<ExpressionValue> {
        let mut span_range = input.span_range();
        match &self {
            UnaryOperation::Neg { token } => span_range.set_start(token.span),
            UnaryOperation::Not { token } => span_range.set_start(token.span),
            UnaryOperation::Cast { target_ident, .. } => span_range.set_end(target_ident.span()),
        };
        input.handle_unary_operation(self.with_output_span_range(span_range))
    }
}

impl Operation for UnaryOperation {
    fn symbol(&self) -> &'static str {
        match self {
            UnaryOperation::Neg { .. } => "-",
            UnaryOperation::Not { .. } => "!",
            UnaryOperation::Cast { .. } => "as",
        }
    }
}

impl HasSpan for UnaryOperation {
    fn span(&self) -> Span {
        match self {
            UnaryOperation::Neg { token } => token.span,
            UnaryOperation::Not { token } => token.span,
            UnaryOperation::Cast { as_token, .. } => as_token.span,
        }
    }
}

pub(super) trait HandleUnaryOperation: Sized {
    fn handle_unary_operation(
        self,
        operation: OutputSpanned<UnaryOperation>,
    ) -> ExecutionResult<ExpressionValue>;
}

#[derive(Clone)]
pub(crate) enum BinaryOperation {
    Paired(PairedBinaryOperation),
    Integer(IntegerBinaryOperation),
}

impl SynParse for BinaryOperation {
    fn parse(input: SynParseStream) -> SynResult<Self> {
        // In line with Syn's BinOp, we use peek instead of lookahead
        // ...I assume for slightly increased performance
        // ...Or because 30 alternative options in the error message is too many
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
            Err(input.error("Expected one of + - * / % && || ^ & | == < <= != >= > << or >>"))
        }
    }
}

impl BinaryOperation {
    pub(super) fn lazy_evaluate(
        &self,
        left: &ExpressionValue,
    ) -> ExecutionResult<Option<ExpressionValue>> {
        match self {
            BinaryOperation::Paired(PairedBinaryOperation::LogicalAnd { .. }) => {
                let bool = left.clone().expect_bool("The left operand to &&")?;
                if !bool.value {
                    Ok(Some(ExpressionValue::Boolean(bool)))
                } else {
                    Ok(None)
                }
            }
            BinaryOperation::Paired(PairedBinaryOperation::LogicalOr { .. }) => {
                let bool = left.clone().expect_bool("The left operand to ||")?;
                if bool.value {
                    Ok(Some(ExpressionValue::Boolean(bool)))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn evaluate(
        &self,
        left: ExpressionValue,
        right: ExpressionValue,
    ) -> ExecutionResult<ExpressionValue> {
        let span_range =
            SpanRange::new_between(left.span_range().start(), right.span_range().end());
        match self {
            BinaryOperation::Paired(operation) => {
                let value_pair = left.expect_value_pair(operation, right)?;
                value_pair
                    .handle_paired_binary_operation(operation.with_output_span_range(span_range))
            }
            BinaryOperation::Integer(operation) => {
                let right = right
                    .into_integer()
                    .ok_or_else(|| self.execution_error("The shift amount must be an integer"))?;
                left.handle_integer_binary_operation(
                    right,
                    operation.with_output_span_range(span_range),
                )
            }
        }
    }
}

impl HasSpanRange for BinaryOperation {
    fn span_range(&self) -> SpanRange {
        match self {
            BinaryOperation::Paired(op) => op.span_range(),
            BinaryOperation::Integer(op) => op.span_range(),
        }
    }
}

impl Operation for BinaryOperation {
    fn symbol(&self) -> &'static str {
        match self {
            BinaryOperation::Paired(paired) => paired.symbol(),
            BinaryOperation::Integer(integer) => integer.symbol(),
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) enum PairedBinaryOperation {
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
pub(crate) enum IntegerBinaryOperation {
    ShiftLeft(Token![<<]),
    ShiftRight(Token![>>]),
}

impl Operation for IntegerBinaryOperation {
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
        operation: OutputSpanned<PairedBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue>;

    fn handle_integer_binary_operation(
        self,
        rhs: ExpressionInteger,
        operation: OutputSpanned<IntegerBinaryOperation>,
    ) -> ExecutionResult<ExpressionValue>;
}

pub(super) trait HandleCreateRange: Sized {
    fn create_range(
        self,
        right: Self,
        range_limits: OutputSpanned<syn::RangeLimits>,
    ) -> Box<dyn Iterator<Item = ExpressionValue> + '_>;
}

impl Operation for syn::RangeLimits {
    fn symbol(&self) -> &'static str {
        match self {
            syn::RangeLimits::HalfOpen(_) => "..",
            syn::RangeLimits::Closed(_) => "..=",
        }
    }
}

impl HasSpanRange for syn::RangeLimits {
    fn span_range(&self) -> SpanRange {
        self.span_range_from_iterating_over_all_tokens()
    }
}
