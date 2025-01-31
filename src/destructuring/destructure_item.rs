use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) enum DestructureItem {
    NoneOutputCommand(Command),
    Variable(DestructureVariable),
    Destructurer(Destructurer),
    ExactPunct(Punct),
    ExactIdent(Ident),
    ExactLiteral(Literal),
    ExactGroup(DestructureGroup),
}

impl DestructureItem {
    /// We provide a stop condition so that some of the items can know when to stop consuming greedily -
    /// notably the flattened command. This allows [!let! #..x = Hello => World] to parse as setting
    /// `x` to `Hello => World` rather than having `#..x` peeking to see it is "up to =" and then only
    /// parsing `Hello` into `x`.
    pub(crate) fn parse_until<C: StopCondition>(input: ParseStream) -> ParseResult<Self> {
        Ok(match detect_preinterpret_grammar(input.cursor()) {
            PeekMatch::Command(Some(CommandOutputKind::None)) => {
                Self::NoneOutputCommand(input.parse()?)
            }
            PeekMatch::Command(_) => {
                return input.parse_err(
                    "Commands which return something are not supported in destructuring positions",
                )
            }
            PeekMatch::GroupedVariable
            | PeekMatch::FlattenedVariable
            | PeekMatch::AppendVariableDestructuring => {
                Self::Variable(DestructureVariable::parse_until::<C>(input)?)
            }
            PeekMatch::Group(_) => Self::ExactGroup(input.parse()?),
            PeekMatch::Destructurer(_) => Self::Destructurer(input.parse()?),
            PeekMatch::Punct(_) => Self::ExactPunct(input.parse_any_punct()?),
            PeekMatch::Literal(_) => Self::ExactLiteral(input.parse()?),
            PeekMatch::Ident(_) => Self::ExactIdent(input.parse_any_ident()?),
            PeekMatch::End => return input.parse_err("Unexpected end"),
        })
    }
}

impl HandleDestructure for DestructureItem {
    fn handle_destructure(
        &self,
        input: ParseStream,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        match self {
            DestructureItem::Variable(variable) => {
                variable.handle_destructure(input, interpreter)?;
            }
            DestructureItem::NoneOutputCommand(command) => {
                let _ = command.clone().interpret_to_new_stream(interpreter)?;
            }
            DestructureItem::Destructurer(destructurer) => {
                destructurer.handle_destructure(input, interpreter)?;
            }
            DestructureItem::ExactPunct(punct) => {
                input.parse_punct_matching(punct.as_char())?;
            }
            DestructureItem::ExactIdent(ident) => {
                input.parse_ident_matching(&ident.to_string())?;
            }
            DestructureItem::ExactLiteral(literal) => {
                input.parse_literal_matching(&literal.to_string())?;
            }
            DestructureItem::ExactGroup(group) => {
                group.handle_destructure(input, interpreter)?;
            }
        }
        Ok(())
    }
}
