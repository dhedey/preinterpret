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

impl Debug for ParserValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parser[{:?}]", self.handle)
    }
}

impl ParserValue {
    pub(crate) fn new(handle: ParserHandle) -> Self {
        Self { handle }
    }
}

impl ValuesEqual for ParserValue {
    /// Parsers are equal if they reference the same handle.
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        if self.handle == other.handle {
            ctx.values_equal()
        } else {
            ctx.leaf_values_not_equal(self, other)
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

impl Shared<ParserValue> {
    pub(crate) fn parser<'i>(
        &self,
        interpreter: &'i mut Interpreter,
    ) -> ExecutionResult<OutputParseStream<'i>> {
        interpreter.parser(self.handle, self.span_range())
    }

    pub(crate) fn parse_with<T>(
        &self,
        interpreter: &mut Interpreter,
        f: impl FnOnce(&mut Interpreter) -> ExecutionResult<T>,
    ) -> ExecutionResult<T> {
        interpreter.parse_with(self.handle, f)
    }
}

fn parser<'a>(
    this: Shared<ParserValue>,
    context: &'a mut MethodCallContext,
) -> ExecutionResult<OutputParseStream<'a>> {
    this.parser(context.interpreter)
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

            [context] fn any_ident(this: Shared<ParserValue>) -> ExecutionResult<Ident> {
                Ok(parser(this, context)?.parse_any_ident()?)
            }

            [context] fn punct(this: Shared<ParserValue>) -> ExecutionResult<Punct> {
                Ok(parser(this, context)?.parse()?)
            }

            [context] fn read(this: Shared<ParserValue>, parse_template: AnyRef<OutputStream>) -> ExecutionResult<()> {
                let this = parser(this, context)?;
                // TODO[parsers] - parse_exact_match doesn't need an output stream
                let mut discarded_output = OutputStream::new();
                parse_template.parse_exact_match(this, &mut discarded_output)
            }

            [context] fn rest(this: Shared<ParserValue>) -> ExecutionResult<OutputStream> {
                let input = parser(this, context)?;
                let mut output = OutputStream::new();
                ParseUntil::End.handle_parse_into(input, &mut output)?;
                Ok(output)
            }

            [context] fn until(this: Shared<ParserValue>, until: OutputStream) -> ExecutionResult<OutputStream> {
                let input = parser(this, context)?;
                let until: ParseUntil = until.parse_as()?;
                let mut output = OutputStream::new();
                until.handle_parse_into(input, &mut output)?;
                Ok(output)
            }

            [context] fn error(this: Shared<ParserValue>, message: String) -> ExecutionResult<()> {
                let parser = parser(this, context)?;
                parser.parse_err(message).map_err(|e| e.into())
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

/// Note: This is very similar to a [`ParseTemplatePattern`], but there, the ident is a *definition*,
/// and used to capture the consumed stream into a variable. Here, the ident is a reference
/// to an existing variable, whose value is expected to be a parser.
pub(crate) struct ParseTemplateLiteral {
    prefix: Token![@],
    parser_reference: VariableReference,
    brackets: Brackets,
    content: ParseTemplateStream,
}

impl ParseSource for ParseTemplateLiteral {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let prefix = input.parse()?;
        let parser_reference = input.parse()?;
        let (brackets, inner) = input.parse_brackets()?;
        let content = ParseTemplateStream::parse_with_span(&inner, brackets.span())?;
        Ok(Self {
            prefix,
            parser_reference,
            brackets,
            content,
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.parser_reference.control_flow_pass(context)?;
        self.content.control_flow_pass(context)
    }
}

impl ParseTemplateLiteral {
    pub(crate) fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue> {
        let parser: Shared<ParserValue> = self
            .parser_reference
            .resolve_shared(interpreter)?
            .resolve_as("The value bound by a consume literal")?;

        parser.parse_with(interpreter, |interpreter| self.content.consume(interpreter))?;

        ownership.map_from_owned(().into_owned_value(self.span_range()))
    }
}

impl HasSpanRange for ParseTemplateLiteral {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.prefix.span, self.brackets.end_span())
    }
}
