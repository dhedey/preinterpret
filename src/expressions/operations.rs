use std::fmt::{Display, Formatter};

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
        let target = TypeIdent::from_ident(&target_ident)?;
        let target = CastTarget::from_source_type(target)?;

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

    pub(super) fn evaluate<T: IntoAnyValue>(
        &self,
        Spanned(input, input_span): Spanned<T>,
    ) -> ExecutionResult<Spanned<ReturnedValue>> {
        let input = input.into_any_value();
        let method = input
            .kind()
            .feature_resolver()
            .resolve_unary_operation(self)
            .ok_or_else(|| {
                self.type_error(format!(
                    "The {} operator is not supported for {} operand",
                    self,
                    input.articled_kind(),
                ))
            })?;
        let input = method
            .argument_ownership
            .map_from_owned(Spanned(input, input_span))?;
        method.execute(Spanned(input, input_span), self)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct CastTarget(pub(crate) AnyValueLeafKind);

impl CastTarget {
    fn from_source_type(s: TypeIdent) -> ParseResult<Self> {
        match s.kind {
            TypeKind::Parent(ParentTypeKind::Integer(_)) => s.parse_err(format!(
                "This type is not supported in cast expressions. Perhaps you want 'as {}'?",
                UntypedIntegerKind.source_type_name()
            )),
            TypeKind::Parent(ParentTypeKind::Float(_)) => s.parse_err(format!(
                "This type is not supported in cast expressions. Perhaps you want 'as {}'?",
                UntypedFloatKind.source_type_name()
            )),
            TypeKind::Leaf(leaf_kind) => Ok(CastTarget(leaf_kind)),
            _ => s.parse_err("This type is not supported in cast expressions"),
        }
    }

    /// Denotes that this cast target for a singleton iterable
    /// (e.g. array or stream)
    pub(crate) fn is_singleton_target(&self) -> bool {
        matches!(
            self.0,
            AnyValueLeafKind::Bool(_)
                | AnyValueLeafKind::Char(_)
                | AnyValueLeafKind::Integer(_)
                | AnyValueLeafKind::Float(_)
        )
    }
}

impl Display for UnaryOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOperation::Neg { .. } => write!(f, "-"),
            UnaryOperation::Not { .. } => write!(f, "!"),
            UnaryOperation::Cast { target, .. } => write!(f, "as {}", target.0.source_type_name()),
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

/// Flattened binary operation enum containing all binary operators.
///
/// This includes:
/// - Arithmetic: Addition (+), Subtraction (-), Multiplication (*), Division (/), Remainder (%)
/// - Logical: LogicalAnd (&&), LogicalOr (||)
/// - Bitwise: BitXor (^), BitAnd (&), BitOr (|), ShiftLeft (<<), ShiftRight (>>)
/// - Comparison: Equal (==), NotEqual (!=), LessThan (<), LessThanOrEqual (<=), GreaterThan (>), GreaterThanOrEqual (>=)
/// - Compound Assignment: AddAssign (+=), SubAssign (-=), MulAssign (*=), DivAssign (/=), RemAssign (%=),
///   BitAndAssign (&=), BitOrAssign (|=), BitXorAssign (^=), ShlAssign (<<=), ShrAssign (>>=)
#[derive(Copy, Clone)]
pub(crate) enum BinaryOperation {
    // Arithmetic operations
    Addition(Token![+]),
    Subtraction(Token![-]),
    Multiplication(Token![*]),
    Division(Token![/]),
    Remainder(Token![%]),
    // Logical operations
    LogicalAnd(Token![&&]),
    LogicalOr(Token![||]),
    // Bitwise operations
    BitXor(Token![^]),
    BitAnd(Token![&]),
    BitOr(Token![|]),
    ShiftLeft(Token![<<]),
    ShiftRight(Token![>>]),
    // Comparison operations
    Equal(Token![==]),
    NotEqual(Token![!=]),
    LessThan(Token![<]),
    LessThanOrEqual(Token![<=]),
    GreaterThan(Token![>]),
    GreaterThanOrEqual(Token![>=]),
    // Compound assignment operations
    AddAssign(Token![+=]),
    SubAssign(Token![-=]),
    MulAssign(Token![*=]),
    DivAssign(Token![/=]),
    RemAssign(Token![%=]),
    BitAndAssign(Token![&=]),
    BitOrAssign(Token![|=]),
    BitXorAssign(Token![^=]),
    ShlAssign(Token![<<=]),
    ShrAssign(Token![>>=]),
}

impl SynParse for BinaryOperation {
    fn parse(input: SynParseStream) -> SynResult<Self> {
        // In line with Syn's BinOp, we use peek instead of lookahead
        // ...I assume for slightly increased performance
        // ...Or because 30 alternative options in the error message is too many
        // NOTE: Order is important here - longer tokens must be checked first,
        // because e.g. Token![+] doesn't check for Spacing::Alone so matches the
        // start of Token![+=]
        // [TODO-performance]: Convert this into a much more efficient parse-tree
        if input.peek(Token![+=]) {
            Ok(Self::AddAssign(input.parse()?))
        } else if input.peek(Token![+]) {
            Ok(Self::Addition(input.parse()?))
        } else if input.peek(Token![-=]) {
            Ok(Self::SubAssign(input.parse()?))
        } else if input.peek(Token![-]) {
            Ok(Self::Subtraction(input.parse()?))
        } else if input.peek(Token![*=]) {
            Ok(Self::MulAssign(input.parse()?))
        } else if input.peek(Token![*]) {
            Ok(Self::Multiplication(input.parse()?))
        } else if input.peek(Token![/=]) {
            Ok(Self::DivAssign(input.parse()?))
        } else if input.peek(Token![/]) {
            Ok(Self::Division(input.parse()?))
        } else if input.peek(Token![%=]) {
            Ok(Self::RemAssign(input.parse()?))
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
        } else if input.peek(Token![<<=]) {
            Ok(Self::ShlAssign(input.parse()?))
        } else if input.peek(Token![<<]) {
            Ok(Self::ShiftLeft(input.parse()?))
        } else if input.peek(Token![>>=]) {
            Ok(Self::ShrAssign(input.parse()?))
        } else if input.peek(Token![>>]) {
            Ok(Self::ShiftRight(input.parse()?))
        } else if input.peek(Token![>]) {
            Ok(Self::GreaterThan(input.parse()?))
        } else if input.peek(Token![<]) {
            Ok(Self::LessThan(input.parse()?))
        } else if input.peek(Token![&=]) {
            Ok(Self::BitAndAssign(input.parse()?))
        } else if input.peek(Token![&]) {
            Ok(Self::BitAnd(input.parse()?))
        } else if input.peek(Token![|=]) {
            Ok(Self::BitOrAssign(input.parse()?))
        } else if input.peek(Token![|]) {
            Ok(Self::BitOr(input.parse()?))
        } else if input.peek(Token![^=]) {
            Ok(Self::BitXorAssign(input.parse()?))
        } else if input.peek(Token![^]) {
            Ok(Self::BitXor(input.parse()?))
        } else {
            Err(input.error("Expected one of + - * / % && || ^ & | == < <= != >= > << >> += -= *= /= %= &= |= ^= <<= or >>="))
        }
    }
}

impl BinaryOperation {
    pub(super) fn lazy_evaluate(
        &self,
        left: Spanned<&AnyValue>,
    ) -> ExecutionResult<Option<AnyValue>> {
        match self {
            BinaryOperation::LogicalAnd { .. } => {
                let bool: Spanned<&bool> = left.resolve_as("The left operand to &&")?;
                if !**bool {
                    Ok(Some((*bool).into_any_value()))
                } else {
                    Ok(None)
                }
            }
            BinaryOperation::LogicalOr { .. } => {
                let bool: Spanned<&bool> = left.resolve_as("The left operand to ||")?;
                if **bool {
                    Ok(Some((*bool).into_any_value()))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    #[allow(unused)]
    pub(crate) fn evaluate<L: IntoAnyValue, R: IntoAnyValue>(
        &self,
        Spanned(left, left_span): Spanned<L>,
        Spanned(right, right_span): Spanned<R>,
    ) -> ExecutionResult<Spanned<ReturnedValue>> {
        let left = left.into_any_value();
        let right = right.into_any_value();
        match left
            .kind()
            .feature_resolver()
            .resolve_binary_operation(self)
        {
            Some(interface) => {
                let left = interface
                    .lhs_ownership
                    .map_from_owned(Spanned(left, left_span))?;
                let right = interface
                    .rhs_ownership
                    .map_from_owned(Spanned(right, right_span))?;
                interface.execute(Spanned(left, left_span), Spanned(right, right_span), self)
            }
            None => self.type_err(format!(
                "The {} operator is not supported for {} operand",
                self.symbolic_description(),
                left.articled_kind(),
            )),
        }
    }
}

impl HasSpanRange for BinaryOperation {
    fn span_range(&self) -> SpanRange {
        match self {
            // Arithmetic
            BinaryOperation::Addition(op) => op.span_range(),
            BinaryOperation::Subtraction(op) => op.span_range(),
            BinaryOperation::Multiplication(op) => op.span_range(),
            BinaryOperation::Division(op) => op.span_range(),
            BinaryOperation::Remainder(op) => op.span_range(),
            // Logical
            BinaryOperation::LogicalAnd(op) => op.span_range(),
            BinaryOperation::LogicalOr(op) => op.span_range(),
            // Bitwise
            BinaryOperation::BitXor(op) => op.span_range(),
            BinaryOperation::BitAnd(op) => op.span_range(),
            BinaryOperation::BitOr(op) => op.span_range(),
            BinaryOperation::ShiftLeft(op) => op.span_range(),
            BinaryOperation::ShiftRight(op) => op.span_range(),
            // Comparison
            BinaryOperation::Equal(op) => op.span_range(),
            BinaryOperation::NotEqual(op) => op.span_range(),
            BinaryOperation::LessThan(op) => op.span_range(),
            BinaryOperation::LessThanOrEqual(op) => op.span_range(),
            BinaryOperation::GreaterThan(op) => op.span_range(),
            BinaryOperation::GreaterThanOrEqual(op) => op.span_range(),
            // Compound assignment
            BinaryOperation::AddAssign(op) => op.span_range(),
            BinaryOperation::SubAssign(op) => op.span_range(),
            BinaryOperation::MulAssign(op) => op.span_range(),
            BinaryOperation::DivAssign(op) => op.span_range(),
            BinaryOperation::RemAssign(op) => op.span_range(),
            BinaryOperation::BitAndAssign(op) => op.span_range(),
            BinaryOperation::BitOrAssign(op) => op.span_range(),
            BinaryOperation::BitXorAssign(op) => op.span_range(),
            BinaryOperation::ShlAssign(op) => op.span_range(),
            BinaryOperation::ShrAssign(op) => op.span_range(),
        }
    }
}

impl Operation for BinaryOperation {
    fn symbolic_description(&self) -> &'static str {
        match self {
            // Arithmetic
            BinaryOperation::Addition { .. } => "+",
            BinaryOperation::Subtraction { .. } => "-",
            BinaryOperation::Multiplication { .. } => "*",
            BinaryOperation::Division { .. } => "/",
            BinaryOperation::Remainder { .. } => "%",
            // Logical
            BinaryOperation::LogicalAnd { .. } => "&&",
            BinaryOperation::LogicalOr { .. } => "||",
            // Bitwise
            BinaryOperation::BitXor { .. } => "^",
            BinaryOperation::BitAnd { .. } => "&",
            BinaryOperation::BitOr { .. } => "|",
            BinaryOperation::ShiftLeft { .. } => "<<",
            BinaryOperation::ShiftRight { .. } => ">>",
            // Comparison
            BinaryOperation::Equal { .. } => "==",
            BinaryOperation::NotEqual { .. } => "!=",
            BinaryOperation::LessThan { .. } => "<",
            BinaryOperation::LessThanOrEqual { .. } => "<=",
            BinaryOperation::GreaterThan { .. } => ">",
            BinaryOperation::GreaterThanOrEqual { .. } => ">=",
            // Compound assignment
            BinaryOperation::AddAssign { .. } => "+=",
            BinaryOperation::SubAssign { .. } => "-=",
            BinaryOperation::MulAssign { .. } => "*=",
            BinaryOperation::DivAssign { .. } => "/=",
            BinaryOperation::RemAssign { .. } => "%=",
            BinaryOperation::BitAndAssign { .. } => "&=",
            BinaryOperation::BitOrAssign { .. } => "|=",
            BinaryOperation::BitXorAssign { .. } => "^=",
            BinaryOperation::ShlAssign { .. } => "<<=",
            BinaryOperation::ShrAssign { .. } => ">>=",
        }
    }
}

pub(super) trait HandleBinaryOperation: Sized + std::fmt::Display + Copy {
    fn type_name() -> &'static str;

    fn binary_overflow_error(
        context: BinaryOperationCallContext,
        lhs: Self,
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

    fn paired_operation<T: From<Self>>(
        self,
        rhs: impl ResolveAs<OptionalSuffix<Self>>,
        context: BinaryOperationCallContext,
        perform_fn: fn(Self, Self) -> Option<Self>,
    ) -> ExecutionResult<T> {
        let lhs = self;
        let rhs = rhs.resolve_as("This operand")?;
        perform_fn(lhs, rhs.0)
            .map(|r| r.into())
            .ok_or_else(|| Self::binary_overflow_error(context, lhs, rhs.0))
    }

    fn paired_operation_no_overflow<T: From<Self>>(
        self,
        rhs: impl ResolveAs<OptionalSuffix<Self>>,
        perform_fn: fn(Self, Self) -> Self,
    ) -> ExecutionResult<T> {
        let lhs = self;
        let rhs = rhs.resolve_as("This operand")?;
        Ok(perform_fn(lhs, rhs.0).into())
    }

    fn paired_comparison(
        self,
        rhs: impl ResolveAs<OptionalSuffix<Self>>,
        compare_fn: fn(Self, Self) -> bool,
    ) -> ExecutionResult<bool> {
        let lhs = self;
        let rhs = rhs.resolve_as("This operand")?;
        Ok(compare_fn(lhs, rhs.0))
    }

    fn shift_operation<O, T: From<O>>(
        self,
        rhs: u32,
        context: BinaryOperationCallContext,
        perform_fn: impl FnOnce(Self, u32) -> Option<O>,
    ) -> ExecutionResult<T> {
        let lhs = self;
        perform_fn(lhs, rhs)
            .map(|r| r.into())
            .ok_or_else(|| Self::binary_overflow_error(context, lhs, rhs))
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
