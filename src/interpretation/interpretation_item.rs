use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) enum InterpretationItem {
    Command(Command),
    GroupedVariable(GroupedVariable),
    FlattenedVariable(FlattenedVariable),
    InterpretationGroup(InterpretationGroup),
    Punct(Punct),
    Ident(Ident),
    Literal(Literal),
}

impl Parse for InterpretationItem {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(match detect_preinterpret_grammar(input.cursor()) {
            PeekMatch::GroupedCommand(_) => InterpretationItem::Command(input.parse()?),
            PeekMatch::FlattenedCommand(_) => InterpretationItem::Command(input.parse()?),
            PeekMatch::Group(_) => InterpretationItem::InterpretationGroup(input.parse()?),
            PeekMatch::GroupedVariable => InterpretationItem::GroupedVariable(input.parse()?),
            PeekMatch::FlattenedVariable => InterpretationItem::FlattenedVariable(input.parse()?),
            PeekMatch::AppendVariableDestructuring | PeekMatch::Destructurer(_) => {
                return input.span().err("Destructurings are not supported here")
            }
            PeekMatch::Punct(_) => InterpretationItem::Punct(input.parse_any_punct()?),
            PeekMatch::Ident(_) => InterpretationItem::Ident(input.parse_any_ident()?),
            PeekMatch::Literal(_) => InterpretationItem::Literal(input.parse()?),
            PeekMatch::End => return input.span().err("Expected some item"),
        })
    }
}

#[allow(unused)]
pub(crate) enum PeekMatch {
    GroupedCommand(Option<CommandKind>),
    FlattenedCommand(Option<CommandKind>),
    GroupedVariable,
    FlattenedVariable,
    AppendVariableDestructuring,
    Destructurer(Option<DestructurerKind>),
    Group(Delimiter),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
    End,
}

pub(crate) fn detect_preinterpret_grammar(cursor: syn::buffer::Cursor) -> PeekMatch {
    // We have to check groups first, so that we handle transparent groups
    // and avoid the self.ignore_none() calls inside cursor
    if let Some((next, delimiter, _, _)) = cursor.any_group() {
        if delimiter == Delimiter::Bracket {
            if let Some((_, next)) = next.punct_matching('!') {
                if let Some((ident, next)) = next.ident() {
                    if next.punct_matching('!').is_some() {
                        return PeekMatch::GroupedCommand(CommandKind::for_ident(&ident));
                    }
                }
                if let Some((_, next)) = next.punct_matching('.') {
                    if let Some((_, next)) = next.punct_matching('.') {
                        if let Some((ident, next)) = next.ident() {
                            if next.punct_matching('!').is_some() {
                                return PeekMatch::FlattenedCommand(CommandKind::for_ident(&ident));
                            }
                        }
                    }
                }
            }
        }
        if delimiter == Delimiter::Parenthesis {
            if let Some((_, next)) = next.punct_matching('!') {
                if let Some((ident, next)) = next.ident() {
                    if next.punct_matching('!').is_some() {
                        return PeekMatch::Destructurer(DestructurerKind::for_ident(&ident));
                    }
                }
            }
        }

        // Ideally we'd like to detect $($tt)* substitutions from macros and interpret them as
        // a Raw (uninterpreted) group, because typically that's what a user would typically intend.
        //
        // You'd think mapping a Delimiter::None to a PeekMatch::RawGroup would be a good way
        // of doing this, but unfortunately this behaviour is very arbitrary and not in a helpful way:
        // => A $tt or $($tt)* is not grouped...
        // => A $literal or $($literal)* _is_ outputted in a group...
        //
        // So this isn't possible. It's unlikely to matter much, and a user can always do:
        // [!raw! $($tt)*] anyway.

        return PeekMatch::Group(delimiter);
    }
    if let Some((_, next)) = cursor.punct_matching('#') {
        if next.ident().is_some() {
            return PeekMatch::GroupedVariable;
        }
        if let Some((_, next)) = next.punct_matching('.') {
            if let Some((_, next)) = next.punct_matching('.') {
                if next.ident().is_some() {
                    return PeekMatch::FlattenedVariable;
                }
                if let Some((_, next)) = next.punct_matching('>') {
                    if next.punct_matching('>').is_some() {
                        return PeekMatch::AppendVariableDestructuring;
                    }
                }
            }
        }
        if let Some((_, next)) = next.punct_matching('>') {
            if next.punct_matching('>').is_some() {
                return PeekMatch::AppendVariableDestructuring;
            }
        }
    }

    match cursor.token_tree() {
        Some((TokenTree::Ident(ident), _)) => PeekMatch::Ident(ident),
        Some((TokenTree::Punct(punct), _)) => PeekMatch::Punct(punct),
        Some((TokenTree::Literal(literal), _)) => PeekMatch::Literal(literal),
        Some((TokenTree::Group(_), _)) => unreachable!("Already covered above"),
        None => PeekMatch::End,
    }
}

impl Interpret for InterpretationItem {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        match self {
            InterpretationItem::Command(command_invocation) => {
                command_invocation.interpret_into(interpreter, output)?;
            }
            InterpretationItem::GroupedVariable(variable) => {
                variable.interpret_into(interpreter, output)?;
            }
            InterpretationItem::FlattenedVariable(variable) => {
                variable.interpret_into(interpreter, output)?;
            }
            InterpretationItem::InterpretationGroup(group) => {
                group.interpret_into(interpreter, output)?;
            }
            InterpretationItem::Punct(punct) => output.push_punct(punct),
            InterpretationItem::Ident(ident) => output.push_ident(ident),
            InterpretationItem::Literal(literal) => output.push_literal(literal),
        }
        Ok(())
    }
}

impl HasSpanRange for InterpretationItem {
    fn span_range(&self) -> SpanRange {
        match self {
            InterpretationItem::Command(command_invocation) => command_invocation.span_range(),
            InterpretationItem::FlattenedVariable(variable) => variable.span_range(),
            InterpretationItem::GroupedVariable(variable) => variable.span_range(),
            InterpretationItem::InterpretationGroup(group) => group.span_range(),
            InterpretationItem::Punct(punct) => punct.span_range(),
            InterpretationItem::Ident(ident) => ident.span_range(),
            InterpretationItem::Literal(literal) => literal.span_range(),
        }
    }
}
