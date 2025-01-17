use super::*;

pub(crate) trait Express: Sized {
    fn add_to_expression(
        self,
        interpreter: &mut Interpreter,
        builder: &mut ExpressionBuilder,
    ) -> Result<()>;

    fn start_expression_builder(self, interpreter: &mut Interpreter) -> Result<ExpressionBuilder> {
        let mut output = ExpressionBuilder::new();
        self.add_to_expression(interpreter, &mut output)?;
        Ok(output)
    }
}

/// This abstraction is a bit ropey...
///
/// Ideally we'd properly handle building expressions into an expression tree,
/// but that requires duplicating some portion of the syn parser for the rust expression tree...
///
/// Instead, to be lazy for now, we interpret the stream at intepretation time to substitute
/// in variables and commands, and then parse the resulting expression with syn.
#[derive(Clone)]
pub(crate) struct ExpressionInput {
    items: Vec<ExpressionItem>,
}

impl Parse for ExpressionInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            // Until we create a proper ExpressionInput parser which builds up a syntax tree
            // then we need to have a way to stop parsing... currently ExpressionInput comes
            // before code blocks or .. in [!range!] so we can break on those.
            // These aren't valid inside expressions we support anyway, so it's good enough for now.
            let item = match detect_preinterpret_grammar(input.cursor()) {
                PeekMatch::Command => ExpressionItem::Command(input.parse()?),
                PeekMatch::GroupedVariable => ExpressionItem::GroupedVariable(input.parse()?),
                PeekMatch::FlattenedVariable => ExpressionItem::FlattenedVariable(input.parse()?),
                PeekMatch::InterpretationGroup(Delimiter::Brace | Delimiter::Bracket) => break,
                PeekMatch::InterpretationGroup(_) => {
                    ExpressionItem::ExpressionGroup(input.parse()?)
                }
                PeekMatch::Other => {
                    if input.cursor().punct_matching('.').is_some() {
                        break;
                    }
                    match input.parse::<TokenTree>()? {
                        TokenTree::Group(_) => {
                            unreachable!(
                                "Should have been already handled by InterpretationGroup above"
                            )
                        }
                        TokenTree::Punct(punct) => ExpressionItem::Punct(punct),
                        TokenTree::Ident(ident) => ExpressionItem::Ident(ident),
                        TokenTree::Literal(literal) => ExpressionItem::Literal(literal),
                    }
                }
            };
            items.push(item);
        }
        if items.is_empty() {
            return input.span().err("Expected an expression");
        }
        Ok(Self { items })
    }
}

impl HasSpanRange for ExpressionInput {
    fn span_range(&self) -> SpanRange {
        SpanRange::new_between(
            self.items.first().unwrap().span(),
            self.items.last().unwrap().span(),
        )
    }
}

impl Express for ExpressionInput {
    fn add_to_expression(
        self,
        interpreter: &mut Interpreter,
        builder: &mut ExpressionBuilder,
    ) -> Result<()> {
        for item in self.items {
            item.add_to_expression(interpreter, builder)?;
        }
        Ok(())
    }
}

impl ExpressionInput {
    pub(crate) fn evaluate(self, interpreter: &mut Interpreter) -> Result<EvaluationOutput> {
        self.start_expression_builder(interpreter)?.evaluate()
    }
}

#[derive(Clone)]
pub(crate) enum ExpressionItem {
    Command(Command),
    GroupedVariable(GroupedVariable),
    FlattenedVariable(FlattenedVariable),
    ExpressionGroup(ExpressionGroup),
    Punct(Punct),
    Ident(Ident),
    Literal(Literal),
}

impl HasSpanRange for ExpressionItem {
    fn span_range(&self) -> SpanRange {
        match self {
            ExpressionItem::Command(command) => command.span_range(),
            ExpressionItem::GroupedVariable(grouped_variable) => grouped_variable.span_range(),
            ExpressionItem::FlattenedVariable(flattened_variable) => {
                flattened_variable.span_range()
            }
            ExpressionItem::ExpressionGroup(expression_group) => expression_group.span_range(),
            ExpressionItem::Punct(punct) => punct.span_range(),
            ExpressionItem::Ident(ident) => ident.span_range(),
            ExpressionItem::Literal(literal) => literal.span_range(),
        }
    }
}

impl Express for ExpressionItem {
    fn add_to_expression(
        self,
        interpreter: &mut Interpreter,
        builder: &mut ExpressionBuilder,
    ) -> Result<()> {
        match self {
            ExpressionItem::Command(command_invocation) => {
                command_invocation.add_to_expression(interpreter, builder)?;
            }
            ExpressionItem::FlattenedVariable(variable) => {
                variable.add_to_expression(interpreter, builder)?;
            }
            ExpressionItem::GroupedVariable(variable) => {
                variable.add_to_expression(interpreter, builder)?;
            }
            ExpressionItem::ExpressionGroup(group) => {
                group.add_to_expression(interpreter, builder)?;
            }
            ExpressionItem::Punct(punct) => builder.push_punct(punct),
            ExpressionItem::Ident(ident) => builder.push_ident(ident),
            ExpressionItem::Literal(literal) => builder.push_literal(literal),
        }
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct ExpressionGroup {
    source_delimiter: Delimiter,
    source_delim_span: DelimSpan,
    content: ExpressionInput,
}

impl Parse for ExpressionGroup {
    fn parse(input: ParseStream) -> Result<Self> {
        let (delimiter, delim_span, content) = input.parse_any_delimiter()?;
        Ok(Self {
            source_delimiter: delimiter,
            source_delim_span: delim_span,
            content: content.parse()?,
        })
    }
}

impl Express for ExpressionGroup {
    fn add_to_expression(
        self,
        interpreter: &mut Interpreter,
        builder: &mut ExpressionBuilder,
    ) -> Result<()> {
        builder.push_expression_group(
            self.content.start_expression_builder(interpreter)?,
            self.source_delimiter,
            self.source_delim_span.join(),
        );
        Ok(())
    }
}

impl HasSpanRange for ExpressionGroup {
    fn span_range(&self) -> SpanRange {
        self.source_delim_span.span_range()
    }
}

#[derive(Clone)]
pub(crate) struct ExpressionBuilder {
    interpreted_stream: InterpretedStream,
}

impl ExpressionBuilder {
    pub(crate) fn new() -> Self {
        let unused_span_range = Span::call_site().span_range();
        Self {
            interpreted_stream: InterpretedStream::new(unused_span_range),
        }
    }

    pub(crate) fn push_literal(&mut self, literal: Literal) {
        self.interpreted_stream.push_literal(literal);
    }

    /// Only true and false make sense, but allow all here and catch others at evaluation time
    pub(crate) fn push_ident(&mut self, ident: Ident) {
        self.interpreted_stream.push_ident(ident);
    }

    pub(crate) fn push_punct(&mut self, punct: Punct) {
        self.interpreted_stream.push_punct(punct);
    }

    pub(crate) fn push_grouped_interpreted_stream(
        &mut self,
        contents: InterpretedStream,
        span: Span,
    ) {
        // Currently using Expr::Parse, it ignores transparent groups, which is
        // a little too permissive.
        // Instead, we use parentheses to ensure that the group has to be a valid
        // expression itself, without being flattened
        self.interpreted_stream
            .push_new_group(contents, Delimiter::Parenthesis, span);
    }

    pub(crate) fn extend_with_interpreted_stream(&mut self, contents: InterpretedStream) {
        self.interpreted_stream.extend(contents);
    }

    pub(crate) fn push_expression_group(
        &mut self,
        contents: Self,
        delimiter: Delimiter,
        span: Span,
    ) {
        self.interpreted_stream
            .push_new_group(contents.interpreted_stream, delimiter, span);
    }

    pub(crate) fn evaluate(self) -> Result<EvaluationOutput> {
        // Parsing into a rust expression is overkill here.
        //
        // In future we could choose to implement a subset of the grammar which we actually can use/need.
        //
        // That said, it's useful for now for two reasons:
        // * Aligning with rust syntax
        // * Saving implementation work, particularly given that syn is likely pulled into most code bases anyway
        //
        // Because of the kind of expressions we're parsing (i.e. no {} allowed),
        // we can get by with parsing it as `Expr::parse` rather than with
        // `Expr::parse_without_eager_brace` or `Expr::parse_with_earlier_boundary_rule`.
        let expression = Expr::parse.parse2(self.interpreted_stream.into_token_stream())?;

        EvaluationTree::build_from(&expression)?.evaluate()
    }
}
