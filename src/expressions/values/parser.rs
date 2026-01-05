use super::*;

define_leaf_type! {
    pub(crate) ParserType => ValueType(ValueContent::Parser),
    content: ParserHandle,
    kind: pub(crate) ParserKind,
    type_name: "parser",
    articled_display_name: "a parser",
    dyn_impls: {},
}

struct ParserHandleValueWrapper(ParserHandle);

impl Debug for ParserHandleValueWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parser[{:?}]", self.0)
    }
}

fn delimiter_from_open_char(c: char) -> Option<Delimiter> {
    match c {
        '(' => Some(Delimiter::Parenthesis),
        '{' => Some(Delimiter::Brace),
        '[' => Some(Delimiter::Bracket),
        _ => None,
    }
}

fn delimiter_from_close_char(c: char) -> Option<Delimiter> {
    match c {
        ')' => Some(Delimiter::Parenthesis),
        '}' => Some(Delimiter::Brace),
        ']' => Some(Delimiter::Bracket),
        _ => None,
    }
}

impl ValuesEqual for ParserHandle {
    /// Parsers are equal if they reference the same handle.
    fn test_equality<C: EqualityContext>(&self, other: &Self, ctx: &mut C) -> C::Result {
        if self == other {
            ctx.values_equal()
        } else {
            ctx.leaf_values_not_equal(
                &ParserHandleValueWrapper(*self),
                &ParserHandleValueWrapper(*other),
            )
        }
    }
}

impl Spanned<Shared<ParserHandle>> {
    pub(crate) fn parser<'i>(
        &self,
        interpreter: &'i mut Interpreter,
    ) -> ExecutionResult<OutputParseStream<'i>> {
        let Spanned(handle, span) = self;
        interpreter.parser(**handle, *span)
    }

    pub(crate) fn parse_with<T>(
        &self,
        interpreter: &mut Interpreter,
        f: impl FnOnce(&mut Interpreter) -> ExecutionResult<T>,
    ) -> ExecutionResult<T> {
        let Spanned(handle, _) = self;
        interpreter.parse_with(**handle, f)
    }
}

fn parser<'a>(
    this: Spanned<Shared<ParserHandle>>,
    context: &'a mut MethodCallContext,
) -> ExecutionResult<OutputParseStream<'a>> {
    this.parser(context.interpreter)
}

define_type_features! {
    impl ParserType,
    pub(crate) mod parser_interface {
        pub(crate) mod methods {
            // GENERAL
            // =======

            [context] fn is_end(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.is_empty())
            }

            // Asserts that the parser has reached the end of input
            [context] fn end(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<()> {
                let parser = parser(this, context)?;
                match parser.is_empty() {
                    true => Ok(()),
                    false => parser.parse_err("unexpected token")?,
                }
            }

            [context] fn token_tree(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<TokenTree> {
                Ok(parser(this, context)?.parse()?)
            }

            [context] fn ident(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<Ident> {
                Ok(parser(this, context)?.parse()?)
            }

            [context] fn any_ident(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<Ident> {
                Ok(parser(this, context)?.parse_any_ident()?)
            }

            [context] fn punct(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<Punct> {
                Ok(parser(this, context)?.parse()?)
            }

            [context] fn read(this: Spanned<Shared<ParserHandle>>, parse_template: AnyRef<OutputStream>) -> ExecutionResult<OutputStream> {
                let this = parser(this, context)?;
                let mut output = OutputStream::new();
                parse_template.parse_exact_match(this, &mut output)?;
                Ok(output)
            }

            [context] fn rest(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<OutputStream> {
                let input = parser(this, context)?;
                let mut output = OutputStream::new();
                ParseUntil::End.handle_parse_into(input, &mut output)?;
                Ok(output)
            }

            [context] fn until(this: Spanned<Shared<ParserHandle>>, until: OutputStream) -> ExecutionResult<OutputStream> {
                let input = parser(this, context)?;
                let until: ParseUntil = until.parse_as()?;
                let mut output = OutputStream::new();
                until.handle_parse_into(input, &mut output)?;
                Ok(output)
            }

            [context] fn error(this: Spanned<Shared<ParserHandle>>, message: String) -> ExecutionResult<()> {
                let parser = parser(this, context)?;
                parser.parse_err(message).map_err(|e| e.into())
            }

            // GROUPS
            // ======

            // Opens a group with the specified delimiter character ('(', '{', or '[').
            // Must be paired with `close`.
            [context] fn open(this: Spanned<Shared<ParserHandle>>, Spanned(delimiter_char, char_span): Spanned<Owned<char>>) -> ExecutionResult<()> {
                let delimiter_char = delimiter_char.into_inner();
                let delimiter = delimiter_from_open_char(delimiter_char)
                    .ok_or_else(|| char_span.value_error(format!(
                        "Invalid open delimiter '{}'. Expected '(', '{{', or '['", delimiter_char
                    )))?;
                this.parse_with(context.interpreter, |interpreter| {
                    interpreter.enter_input_group(Some(delimiter))?;
                    Ok(())
                })
            }

            // Closes the current group. Must be paired with a prior `open`.
            // The close character must match: ')' for '(', '}' for '{', ']' for '['
            [context] fn close(this: Spanned<Shared<ParserHandle>>, Spanned(delimiter_char, char_span): Spanned<Owned<char>>) -> ExecutionResult<()> {
                let delimiter_char = delimiter_char.into_inner();
                let expected_delimiter = delimiter_from_close_char(delimiter_char)
                    .ok_or_else(|| char_span.value_error(format!(
                        "Invalid close delimiter '{}'. Expected ')', '}}', or ']'", delimiter_char
                    )))?;
                this.parse_with(context.interpreter, |interpreter| {
                    // Check if there's a group to close first
                    if !interpreter.has_active_input_group() {
                        return Err(char_span.value_error(format!(
                            "attempting to close '{}' isn't valid, because there is no open group",
                            expected_delimiter.description_of_close()
                        )));
                    }
                    if !interpreter.input().is_empty() {
                        return interpreter.input().parse_err(format!(
                            "expected '{}'", expected_delimiter.description_of_close()
                        ))?;
                    }
                    interpreter.exit_input_group(Some(expected_delimiter))?;
                    Ok(())
                })
            }

            // LITERALS
            // ========

            [context] fn is_literal(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.cursor().literal().is_some())
            }

            [context] fn literal(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<OutputStream> {
                let literal = parser(this, context)?.parse()?;
                Ok(OutputStream::new_with(|s| s.push_literal(literal)))
            }

            [context] fn inferred_literal(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<Value> {
                let literal = parser(this, context)?.parse()?;
                Ok(Value::for_literal(literal).into_value())
            }

            [context] fn is_char(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.peek(syn::LitChar))
            }

            [context] fn char_literal(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<OutputStream> {
                let char: syn::LitChar = parser(this, context)?.parse()?;
                Ok(OutputStream::new_with(|s| s.push_tokens(char)))
            }

            [context] fn char(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<char> {
                let char: syn::LitChar = parser(this, context)?.parse()?;
                Ok(char.value())
            }

            [context] fn is_string(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.peek(syn::LitStr))
            }

            [context] fn string_literal(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<OutputStream> {
                let string: syn::LitStr = parser(this, context)?.parse()?;
                Ok(OutputStream::new_with(|s| s.push_tokens(string)))
            }

            [context] fn string(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<String> {
                let string: syn::LitStr = parser(this, context)?.parse()?;
                Ok(string.value())
            }

            [context] fn is_integer(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.peek(syn::LitInt))
            }

            [context] fn integer_literal(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<OutputStream> {
                let integer: syn::LitInt = parser(this, context)?.parse()?;
                Ok(OutputStream::new_with(|s| s.push_tokens(integer)))
            }

            [context] fn integer(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<IntegerValue> {
                let integer: syn::LitInt = parser(this, context)?.parse()?;
                Ok(IntegerValue::for_litint(&integer)?.into_inner())
            }

            [context] fn is_float(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<bool> {
                Ok(parser(this, context)?.peek(syn::LitFloat))
            }

            [context] fn float_literal(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<OutputStream> {
                let float: syn::LitFloat = parser(this, context)?.parse()?;
                Ok(OutputStream::new_with(|s| s.push_tokens(float)))
            }

            [context] fn float(this: Spanned<Shared<ParserHandle>>) -> ExecutionResult<FloatValue> {
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
    ParserType,
    (value, context) -> ParserHandle {
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

impl Evaluate for ParseTemplateLiteral {
    fn evaluate(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<RequestedValue> {
        let parser: Shared<ParserHandle> = Spanned(
            self.parser_reference.resolve_shared(interpreter)?,
            self.parser_reference.span_range(),
        )
        .resolve_as("The value bound by a consume literal")?;

        let parser = parser.spanned(self.parser_reference.span_range());

        parser.parse_with(interpreter, |interpreter| self.content.consume(interpreter))?;

        ownership
            .map_from_owned(Spanned(().into_owned_value(), self.span_range()))
            .map(|spanned| spanned.0)
    }
}

impl HasSpanRange for ParseTemplateLiteral {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(self.prefix.span, self.brackets.end_span())
    }
}
