use super::*;

#[derive(Clone)]
pub(crate) struct ParserValue {
    handle: ParserHandle,
}

impl HasValueKind for ParserValue {
    type SpecificKind = ValueKind;

    fn kind(&self) -> ValueKind {
        ValueKind::Parser
    }
}

impl ParserValue {
    pub(crate) fn new(handle: ParserHandle) -> Self {
        Self { handle }
    }
}

impl ValuesEqual for ParserValue {
    /// Parsers are equal if they reference the same handle.
    fn values_equal<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        if self.handle == other.handle {
            ctx.equal()
        } else {
            ctx.not_equal(self, other)
        }
    }
}

impl IntoValue for ParserValue {
    fn into_value(self) -> Value {
        Value::Parser(self)
    }
}

impl IntoValue for ParserHandle {
    fn into_value(self) -> Value {
        ParserValue::new(self).into_value()
    }
}

fn parser<'a>(
    this: Shared<ParserValue>,
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

            [context] fn is_end(this: Shared<ParserValue>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.is_empty())
            }

            // Asserts that the parser has reached the end of input
            [context] fn end(this: Shared<ParserValue>) -> ExecutionResult<()> {
                let parser = parser(this, context)?;
                match parser.is_empty() {
                    true => Ok(()),
                    false => parser.parse_err("unexpected token")?,
                }
            }

            [context] fn token_tree(this: Shared<ParserValue>) -> ExecutionResult<TokenTree> {
                Ok(parser(this, context)?.parse()?)
            }

            [context] fn ident(this: Shared<ParserValue>) -> ExecutionResult<Ident> {
                Ok(parser(this, context)?.parse()?)
            }

            [context] fn punct(this: Shared<ParserValue>) -> ExecutionResult<Punct> {
                Ok(parser(this, context)?.parse()?)
            }

            // LITERALS
            // ========

            [context] fn is_literal(this: Shared<ParserValue>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.cursor().literal().is_some())
            }

            [context] fn literal(this: Shared<ParserValue>) -> ExecutionResult<OutputStream> {
                let literal = parser(this, context)?.parse()?;
                Ok(OutputStream::new_with(|s| s.push_literal(literal)))
            }

            [context] fn inferred_literal(this: Shared<ParserValue>) -> ExecutionResult<Value> {
                let literal = parser(this, context)?.parse()?;
                Ok(Value::for_literal(literal).into_value())
            }

            [context] fn is_char(this: Shared<ParserValue>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.peek(syn::LitChar))
            }

            [context] fn char_literal(this: Shared<ParserValue>) -> ExecutionResult<OutputStream> {
                let char: syn::LitChar = parser(this, context)?.parse()?;
                Ok(OutputStream::new_with(|s| s.push_tokens(char)))
            }

            [context] fn char(this: Shared<ParserValue>) -> ExecutionResult<char> {
                let char: syn::LitChar = parser(this, context)?.parse()?;
                Ok(char.value())
            }

            [context] fn is_string(this: Shared<ParserValue>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.peek(syn::LitStr))
            }

            [context] fn string_literal(this: Shared<ParserValue>) -> ExecutionResult<OutputStream> {
                let string: syn::LitStr = parser(this, context)?.parse()?;
                Ok(OutputStream::new_with(|s| s.push_tokens(string)))
            }

            [context] fn string(this: Shared<ParserValue>) -> ExecutionResult<String> {
                let string: syn::LitStr = parser(this, context)?.parse()?;
                Ok(string.value())
            }

            [context] fn is_integer(this: Shared<ParserValue>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.peek(syn::LitInt))
            }

            [context] fn integer_literal(this: Shared<ParserValue>) -> ExecutionResult<OutputStream> {
                let integer: syn::LitInt = parser(this, context)?.parse()?;
                Ok(OutputStream::new_with(|s| s.push_tokens(integer)))
            }

            [context] fn integer(this: Shared<ParserValue>) -> ExecutionResult<IntegerValue> {
                let integer: syn::LitInt = parser(this, context)?.parse()?;
                Ok(IntegerValue::for_litint(&integer)?.into_inner())
            }

            [context] fn is_float(this: Shared<ParserValue>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.peek(syn::LitFloat))
            }

            [context] fn float_literal(this: Shared<ParserValue>) -> ExecutionResult<OutputStream> {
                let float: syn::LitFloat = parser(this, context)?.parse()?;
                Ok(OutputStream::new_with(|s| s.push_tokens(float)))
            }

            [context] fn float(this: Shared<ParserValue>) -> ExecutionResult<FloatValue> {
                let float: syn::LitFloat = parser(this, context)?.parse()?;
                Ok(FloatValue::for_litfloat(&float)?.into_inner())
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
    (value, context) -> ParserValue {
        match value {
            Value::Parser(value) => Ok(value),
            other => context.err("a parser", other),
        }
    }
}

impl IntoValue for TokenTree {
    fn into_value(self) -> Value {
        OutputStream::new_with(|s| s.push_raw_token_tree(self)).into_value()
    }
}

impl IntoValue for Ident {
    fn into_value(self) -> Value {
        OutputStream::new_with(|s| s.push_ident(self)).into_value()
    }
}

impl IntoValue for Punct {
    fn into_value(self) -> Value {
        OutputStream::new_with(|s| s.push_punct(self)).into_value()
    }
}

impl IntoValue for Literal {
    fn into_value(self) -> Value {
        OutputStream::new_with(|s| s.push_literal(self)).into_value()
    }
}
