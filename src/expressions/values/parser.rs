use super::*;

#[derive(Clone)]
pub(crate) struct ParserExpression {
    handle: ParserHandle,
}

impl HasValueType for ParserExpression {
    fn value_type(&self) -> &'static str {
        "parser"
    }
}

impl ParserExpression {
    pub(crate) fn new(handle: ParserHandle) -> Self {
        Self { handle }
    }
}

impl ToExpressionValue for ParserExpression {
    fn into_value(self) -> ExpressionValue {
        ExpressionValue::Parser(self)
    }
}

impl ToExpressionValue for ParserHandle {
    fn into_value(self) -> ExpressionValue {
        ParserExpression::new(self).into_value()
    }
}

fn parser<'a>(
    this: Shared<ParserExpression>,
    context: &'a mut MethodCallContext,
) -> ExecutionResult<OutputParseStream<'a>> {
    context
        .interpreter
        .parser(this.as_spanned().map(|e, _| e.handle))
}

define_interface! {
    struct ParserTypeData,
    parent: ValueTypeData,
    pub(crate) mod parser_interface {
        pub(crate) mod methods {
            // GENERAL
            // =======

            [context] fn is_end(this: Shared<ParserExpression>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.is_empty())
            }

            // Asserts that the parser has reached the end of input
            [context] fn end(this: Shared<ParserExpression>) -> ExecutionResult<()> {
                let parser = parser(this, context)?;
                match parser.is_empty() {
                    true => Ok(()),
                    false => parser.parse_err("unexpected token")?,
                }
            }

            [context] fn token_tree(this: Shared<ParserExpression>) -> ExecutionResult<TokenTree> {
                Ok(parser(this, context)?.parse()?)
            }

            [context] fn ident(this: Shared<ParserExpression>) -> ExecutionResult<Ident> {
                Ok(parser(this, context)?.parse()?)
            }

            [context] fn punct(this: Shared<ParserExpression>) -> ExecutionResult<Punct> {
                Ok(parser(this, context)?.parse()?)
            }

            // LITERALS
            // ========

            [context] fn is_literal(this: Shared<ParserExpression>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.cursor().literal().is_some())
            }

            [context] fn literal(this: Shared<ParserExpression>) -> ExecutionResult<OutputStream> {
                let literal = parser(this, context)?.parse()?;
                Ok(OutputStream::new_with(|s| s.push_literal(literal)))
            }

            [context] fn inferred_literal(this: Shared<ParserExpression>) -> ExecutionResult<ExpressionValue> {
                let literal = parser(this, context)?.parse()?;
                Ok(ExpressionValue::for_literal(literal).into_value())
            }

            [context] fn is_char(this: Shared<ParserExpression>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.peek(syn::LitChar))
            }

            [context] fn char_literal(this: Shared<ParserExpression>) -> ExecutionResult<OutputStream> {
                let char: syn::LitChar = parser(this, context)?.parse()?;
                Ok(OutputStream::new_with(|s| s.push_tokens(char)))
            }

            [context] fn char(this: Shared<ParserExpression>) -> ExecutionResult<char> {
                let char: syn::LitChar = parser(this, context)?.parse()?;
                Ok(char.value())
            }

            [context] fn is_string(this: Shared<ParserExpression>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.peek(syn::LitStr))
            }

            [context] fn string_literal(this: Shared<ParserExpression>) -> ExecutionResult<OutputStream> {
                let string: syn::LitStr = parser(this, context)?.parse()?;
                Ok(OutputStream::new_with(|s| s.push_tokens(string)))
            }

            [context] fn string(this: Shared<ParserExpression>) -> ExecutionResult<String> {
                let string: syn::LitStr = parser(this, context)?.parse()?;
                Ok(string.value())
            }

            [context] fn is_integer(this: Shared<ParserExpression>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.peek(syn::LitInt))
            }

            [context] fn integer_literal(this: Shared<ParserExpression>) -> ExecutionResult<OutputStream> {
                let integer: syn::LitInt = parser(this, context)?.parse()?;
                Ok(OutputStream::new_with(|s| s.push_tokens(integer)))
            }

            [context] fn integer(this: Shared<ParserExpression>) -> ExecutionResult<IntegerExpression> {
                let integer: syn::LitInt = parser(this, context)?.parse()?;
                Ok(IntegerExpression::for_litint(&integer)?.into_inner())
            }

            [context] fn is_float(this: Shared<ParserExpression>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.peek(syn::LitFloat))
            }

            [context] fn float_literal(this: Shared<ParserExpression>) -> ExecutionResult<OutputStream> {
                let float: syn::LitFloat = parser(this, context)?.parse()?;
                Ok(OutputStream::new_with(|s| s.push_tokens(float)))
            }

            [context] fn float(this: Shared<ParserExpression>) -> ExecutionResult<FloatExpression> {
                let float: syn::LitFloat = parser(this, context)?.parse()?;
                Ok(FloatExpression::for_litfloat(&float)?.into_inner())
            }
        }
        pub(crate) mod unary_operations {
        }
        pub(crate) mod binary_operations {}
        interface_items {
        }
    }
}

impl_resolvable_argument_for! {
    ParserTypeData,
    (value, context) -> ParserExpression {
        match value {
            ExpressionValue::Parser(value) => Ok(value),
            other => context.err("parser", other),
        }
    }
}

impl ToExpressionValue for TokenTree {
    fn into_value(self) -> ExpressionValue {
        OutputStream::new_with(|s| s.push_raw_token_tree(self)).into_value()
    }
}

impl ToExpressionValue for Ident {
    fn into_value(self) -> ExpressionValue {
        OutputStream::new_with(|s| s.push_ident(self)).into_value()
    }
}

impl ToExpressionValue for Punct {
    fn into_value(self) -> ExpressionValue {
        OutputStream::new_with(|s| s.push_punct(self)).into_value()
    }
}

impl ToExpressionValue for Literal {
    fn into_value(self) -> ExpressionValue {
        OutputStream::new_with(|s| s.push_literal(self)).into_value()
    }
}
