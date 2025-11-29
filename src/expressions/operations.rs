use super::*;

pub(crate) trait Operation: HasSpanRange {
    fn symbolic_description(&self) -> &'static str;
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
pub(crate) enum UnaryOperation {
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
        let target = CastTarget::from_str(target_ident.to_string().as_str()).map_err(|()| {
            target_ident.parse_error("This type is not supported in cast expressions")
        })?;

        Ok(Self::Cast {
            as_token,
            target,
            target_ident,
        })
    }

    pub(super) fn output_span_range(&self, mut operand_span_range: SpanRange) -> SpanRange {
        match self {
            UnaryOperation::Neg { token } => operand_span_range.set_start(token.span),
            UnaryOperation::Not { token } => operand_span_range.set_start(token.span),
            UnaryOperation::Cast { target_ident, .. } => {
                operand_span_range.set_end(target_ident.span())
            }
        };
        operand_span_range
    }

    pub(super) fn evaluate<T: ToExpressionValue>(
        &self,
        input: Owned<T>,
    ) -> ExecutionResult<ResolvedValue> {
        let input = input.into_owned_value();
        let method = input.kind().resolve_unary_operation(self).ok_or_else(|| {
            self.type_error(format!(
                "The {} operator is not supported for {} operand",
                self.symbolic_description(),
                input.articled_value_type(),
            ))
        })?;
        let input = method.argument_ownership.map_from_owned(input)?;
        method.execute(input, self)
    }
}

#[derive(Copy, Clone)]
pub(crate) enum CastTarget {
    Integer(IntegerKind),
    Float(FloatKind),
    Boolean,
    String,
    Char,
    Stream,
}

impl FromStr for CastTarget {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
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
            "string" => CastTarget::String,
            _ => return Err(()),
        })
    }
}

impl CastTarget {
    fn symbolic_description(&self) -> &'static str {
        match self {
            CastTarget::Integer(IntegerKind::Untyped) => "as int",
            CastTarget::Integer(IntegerKind::U8) => "as u8",
            CastTarget::Integer(IntegerKind::U16) => "as u16",
            CastTarget::Integer(IntegerKind::U32) => "as u32",
            CastTarget::Integer(IntegerKind::U64) => "as u64",
            CastTarget::Integer(IntegerKind::U128) => "as u128",
            CastTarget::Integer(IntegerKind::Usize) => "as usize",
            CastTarget::Integer(IntegerKind::I8) => "as i8",
            CastTarget::Integer(IntegerKind::I16) => "as i16",
            CastTarget::Integer(IntegerKind::I32) => "as i32",
            CastTarget::Integer(IntegerKind::I64) => "as i64",
            CastTarget::Integer(IntegerKind::I128) => "as i128",
            CastTarget::Integer(IntegerKind::Isize) => "as isize",
            CastTarget::Float(FloatKind::Untyped) => "as float",
            CastTarget::Float(FloatKind::F32) => "as f32",
            CastTarget::Float(FloatKind::F64) => "as f64",
            CastTarget::Boolean => "as bool",
            CastTarget::String => "as string",
            CastTarget::Char => "as char",
            CastTarget::Stream => "as stream",
        }
    }
}

impl Operation for UnaryOperation {
    fn symbolic_description(&self) -> &'static str {
        match self {
            UnaryOperation::Neg { .. } => "-",
            UnaryOperation::Not { .. } => "!",
            UnaryOperation::Cast { target, .. } => target.symbolic_description(),
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

#[derive(Copy, Clone)]
pub(crate) enum BinaryOperation {
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
    ShiftLeft(Token![<<]),
    ShiftRight(Token![>>]),
}

impl SynParse for BinaryOperation {
    fn parse(input: SynParseStream) -> SynResult<Self> {
        // In line with Syn's BinOp, we use peek instead of lookahead
        // ...I assume for slightly increased performance
        // ...Or because 30 alternative options in the error message is too many
        if input.peek(Token![+]) {
            Ok(Self::Addition(input.parse()?))
        } else if input.peek(Token![-]) {
            Ok(Self::Subtraction(input.parse()?))
        } else if input.peek(Token![*]) {
            Ok(Self::Multiplication(input.parse()?))
        } else if input.peek(Token![/]) {
            Ok(Self::Division(input.parse()?))
        } else if input.peek(Token![%]) {
            Ok(Self::Remainder(input.parse()?))
        } else if input.peek(Token![&&]) {
            Ok(Self::LogicalAnd(input.parse()?))
        } else if input.peek(Token![||]) {
            Ok(Self::LogicalOr(input.parse()?))
        } else if input.peek(Token![==]) {
            Ok(Self::Equal(input.parse()?))
        } else if input.peek(Token![!=]) {
            Ok(Self::NotEqual(input.parse()?))
        } else if input.peek(Token![>=]) {
            Ok(Self::GreaterThanOrEqual(input.parse()?))
        } else if input.peek(Token![<=]) {
            Ok(Self::LessThanOrEqual(input.parse()?))
        } else if input.peek(Token![<<]) {
            Ok(Self::ShiftLeft(input.parse()?))
        } else if input.peek(Token![>>]) {
            Ok(Self::ShiftRight(input.parse()?))
        } else if input.peek(Token![>]) {
            Ok(Self::GreaterThan(input.parse()?))
        } else if input.peek(Token![<]) {
            Ok(Self::LessThan(input.parse()?))
        } else if input.peek(Token![&]) {
            Ok(Self::BitAnd(input.parse()?))
        } else if input.peek(Token![|]) {
            Ok(Self::BitOr(input.parse()?))
        } else if input.peek(Token![^]) {
            Ok(Self::BitXor(input.parse()?))
        } else {
            Err(input.error("Expected one of + - * / % && || ^ & | == < <= != >= > << or >>"))
        }
    }
}

impl BinaryOperation {
    pub(super) fn lazy_evaluate(
        &self,
        left: Spanned<&ExpressionValue>,
    ) -> ExecutionResult<Option<OwnedValue>> {
        match self {
            BinaryOperation::LogicalAnd { .. } => {
                let bool: Spanned<&bool> = left.resolve_as("The left operand to &&")?;
                if !*bool.value {
                    Ok(Some(bool.value.into_owned_value(bool.span_range)))
                } else {
                    Ok(None)
                }
            }
            BinaryOperation::LogicalOr { .. } => {
                let bool: Spanned<&bool> = left.resolve_as("The left operand to ||")?;
                if *bool.value {
                    Ok(Some(bool.value.into_owned_value(bool.span_range)))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn evaluate<L: ToExpressionValue, R: ToExpressionValue>(
        &self,
        left: Owned<L>,
        right: Owned<R>,
    ) -> ExecutionResult<ResolvedValue> {
        let left = left.into_owned_value();
        let right = right.into_owned_value();
        let interface = left.kind().resolve_binary_operation(self).ok_or_else(|| {
            self.type_error(format!(
                "The {} operator is not supported for {} operand",
                self.symbolic_description(),
                left.articled_value_type(),
            ))
        })?;
        let left = interface.lhs_ownership.map_from_owned(left)?;
        let right = interface.rhs_ownership.map_from_owned(right)?;
        interface.execute(left, right, self)
    }
}

impl HasSpanRange for BinaryOperation {
    fn span_range(&self) -> SpanRange {
        match self {
            BinaryOperation::Addition(t) => t.span_range(),
            BinaryOperation::Subtraction(t) => t.span_range(),
            BinaryOperation::Multiplication(t) => t.span_range(),
            BinaryOperation::Division(t) => t.span_range(),
            BinaryOperation::Remainder(t) => t.span_range(),
            BinaryOperation::LogicalAnd(t) => t.span_range(),
            BinaryOperation::LogicalOr(t) => t.span_range(),
            BinaryOperation::BitXor(t) => t.span_range(),
            BinaryOperation::BitAnd(t) => t.span_range(),
            BinaryOperation::BitOr(t) => t.span_range(),
            BinaryOperation::Equal(t) => t.span_range(),
            BinaryOperation::LessThan(t) => t.span_range(),
            BinaryOperation::LessThanOrEqual(t) => t.span_range(),
            BinaryOperation::NotEqual(t) => t.span_range(),
            BinaryOperation::GreaterThanOrEqual(t) => t.span_range(),
            BinaryOperation::GreaterThan(t) => t.span_range(),
            BinaryOperation::ShiftLeft(t) => t.span_range(),
            BinaryOperation::ShiftRight(t) => t.span_range(),
        }
    }
}

impl Operation for BinaryOperation {
    fn symbolic_description(&self) -> &'static str {
        match self {
            BinaryOperation::Addition { .. } => "+",
            BinaryOperation::Subtraction { .. } => "-",
            BinaryOperation::Multiplication { .. } => "*",
            BinaryOperation::Division { .. } => "/",
            BinaryOperation::Remainder { .. } => "%",
            BinaryOperation::LogicalAnd { .. } => "&&",
            BinaryOperation::LogicalOr { .. } => "||",
            BinaryOperation::BitXor { .. } => "^",
            BinaryOperation::BitAnd { .. } => "&",
            BinaryOperation::BitOr { .. } => "|",
            BinaryOperation::Equal { .. } => "==",
            BinaryOperation::LessThan { .. } => "<",
            BinaryOperation::LessThanOrEqual { .. } => "<=",
            BinaryOperation::NotEqual { .. } => "!=",
            BinaryOperation::GreaterThanOrEqual { .. } => ">=",
            BinaryOperation::GreaterThan { .. } => ">",
            BinaryOperation::ShiftLeft { .. } => "<<",
            BinaryOperation::ShiftRight { .. } => ">>",
        }
    }
}

/// Helper trait for implementing binary operations with overflow checking.
/// This provides common utilities for the new binary operations interface.
pub(super) trait BinaryOperationHelper: Sized + std::fmt::Display + Copy {
    fn type_name() -> &'static str;

    fn binary_overflow_error(
        context: BinaryOperationCallContext,
        lhs: impl std::fmt::Display,
        rhs: impl std::fmt::Display,
    ) -> ExecutionInterrupt {
        context.error(format!(
            "The {} operation {} {} {} overflowed",
            Self::type_name(),
            lhs,
            context.operation.symbolic_description(),
            rhs
        ))
    }

    fn paired_operation(
        lhs: Self,
        rhs: Self,
        context: BinaryOperationCallContext,
        perform_fn: fn(Self, Self) -> Option<Self>,
    ) -> ExecutionResult<Self> {
        perform_fn(lhs, rhs).ok_or_else(|| Self::binary_overflow_error(context, lhs, rhs))
    }
}

impl Operation for syn::RangeLimits {
    fn symbolic_description(&self) -> &'static str {
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

#[derive(Clone, Copy)]
pub(crate) enum CompoundAssignmentOperation {
    Add(Token![+=]),
    Sub(Token![-=]),
    Mul(Token![*=]),
    Div(Token![/=]),
    Rem(Token![%=]),
    BitAnd(Token![&=]),
    BitOr(Token![|=]),
    BitXor(Token![^=]),
    Shl(Token![<<=]),
    Shr(Token![>>=]),
}

impl SynParse for CompoundAssignmentOperation {
    fn parse(input: SynParseStream) -> SynResult<Self> {
        // In line with Syn's BinOp, we use peek instead of lookahead
        // ...I assume for slightly increased performance
        // ...Or because 30 alternative options in the error message is too many
        if input.peek(Token![+=]) {
            Ok(Self::Add(input.parse()?))
        } else if input.peek(Token![-=]) {
            Ok(Self::Sub(input.parse()?))
        } else if input.peek(Token![*=]) {
            Ok(Self::Mul(input.parse()?))
        } else if input.peek(Token![/=]) {
            Ok(Self::Div(input.parse()?))
        } else if input.peek(Token![%=]) {
            Ok(Self::Rem(input.parse()?))
        } else if input.peek(Token![&=]) {
            Ok(Self::BitAnd(input.parse()?))
        } else if input.peek(Token![|=]) {
            Ok(Self::BitOr(input.parse()?))
        } else if input.peek(Token![^=]) {
            Ok(Self::BitXor(input.parse()?))
        } else if input.peek(Token![<<=]) {
            Ok(Self::Shl(input.parse()?))
        } else if input.peek(Token![>>=]) {
            Ok(Self::Shr(input.parse()?))
        } else {
            Err(input.error("Expected one of += -= *= /= %= &= |= ^= <<= or >>="))
        }
    }
}

impl CompoundAssignmentOperation {
    pub(crate) fn to_binary(self) -> BinaryOperation {
        match self {
            CompoundAssignmentOperation::Add(token) => {
                let token = create_single_token('+', token.spans[0]);
                BinaryOperation::Addition(token)
            }
            CompoundAssignmentOperation::Sub(token) => {
                let token = create_single_token('-', token.spans[0]);
                BinaryOperation::Subtraction(token)
            }
            CompoundAssignmentOperation::Mul(token) => {
                let token = create_single_token('*', token.spans[0]);
                BinaryOperation::Multiplication(token)
            }
            CompoundAssignmentOperation::Div(token) => {
                let token = create_single_token('/', token.spans[0]);
                BinaryOperation::Division(token)
            }
            CompoundAssignmentOperation::Rem(token) => {
                let token = create_single_token('%', token.spans[0]);
                BinaryOperation::Remainder(token)
            }
            CompoundAssignmentOperation::BitAnd(token) => {
                let token = create_single_token('&', token.spans[0]);
                BinaryOperation::BitAnd(token)
            }
            CompoundAssignmentOperation::BitOr(token) => {
                let token = create_single_token('^', token.spans[0]);
                BinaryOperation::BitOr(token)
            }
            CompoundAssignmentOperation::BitXor(token) => {
                let token = create_single_token('|', token.spans[0]);
                BinaryOperation::BitXor(token)
            }
            CompoundAssignmentOperation::Shl(token) => {
                let token = create_double_token('<', token.spans[0], '<', token.spans[1]);
                BinaryOperation::ShiftLeft(token)
            }
            CompoundAssignmentOperation::Shr(token) => {
                let token = create_double_token('>', token.spans[0], '>', token.spans[1]);
                BinaryOperation::ShiftRight(token)
            }
        }
    }
}

fn create_single_token<T: SynParse>(char: char, span: Span) -> T {
    let stream = Punct::new(char, Spacing::Alone)
        .with_span(span)
        .to_token_stream();
    T::parse.parse2(stream).unwrap()
}

fn create_double_token<T: SynParse>(char1: char, span1: Span, char2: char, span2: Span) -> T {
    let mut stream = TokenStream::new();
    Punct::new(char1, Spacing::Joint)
        .with_span(span1)
        .to_tokens(&mut stream);
    Punct::new(char2, Spacing::Alone)
        .with_span(span2)
        .to_tokens(&mut stream);
    T::parse.parse2(stream).unwrap()
}

impl Operation for CompoundAssignmentOperation {
    fn symbolic_description(&self) -> &'static str {
        match self {
            CompoundAssignmentOperation::Add(_) => "+=",
            CompoundAssignmentOperation::Sub(_) => "-=",
            CompoundAssignmentOperation::Mul(_) => "*=",
            CompoundAssignmentOperation::Div(_) => "/=",
            CompoundAssignmentOperation::Rem(_) => "%=",
            CompoundAssignmentOperation::BitAnd(_) => "&=",
            CompoundAssignmentOperation::BitOr(_) => "|=",
            CompoundAssignmentOperation::BitXor(_) => "^=",
            CompoundAssignmentOperation::Shl(_) => "<<=",
            CompoundAssignmentOperation::Shr(_) => ">>=",
        }
    }
}

impl HasSpanRange for CompoundAssignmentOperation {
    fn span_range(&self) -> SpanRange {
        match self {
            CompoundAssignmentOperation::Add(op) => op.span_range(),
            CompoundAssignmentOperation::Sub(op) => op.span_range(),
            CompoundAssignmentOperation::Mul(op) => op.span_range(),
            CompoundAssignmentOperation::Div(op) => op.span_range(),
            CompoundAssignmentOperation::Rem(op) => op.span_range(),
            CompoundAssignmentOperation::BitAnd(op) => op.span_range(),
            CompoundAssignmentOperation::BitOr(op) => op.span_range(),
            CompoundAssignmentOperation::BitXor(op) => op.span_range(),
            CompoundAssignmentOperation::Shl(op) => op.span_range(),
            CompoundAssignmentOperation::Shr(op) => op.span_range(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct PropertyAccess {
    pub(super) dot: Token![.],
    pub(crate) property: Ident,
}

impl HasSpanRange for PropertyAccess {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.dot.span, self.property.span())
    }
}

#[derive(Clone)]
pub(crate) struct MethodAccess {
    pub(super) dot: Token![.],
    pub(crate) method: Ident,
    pub(crate) parentheses: Parentheses,
}

impl HasSpanRange for MethodAccess {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.dot.span, self.parentheses.span())
    }
}

#[derive(Copy, Clone)]
pub(crate) struct IndexAccess {
    pub(crate) brackets: Brackets,
}

impl HasSpan for IndexAccess {
    fn span(&self) -> Span {
        self.brackets.join()
    }
}
