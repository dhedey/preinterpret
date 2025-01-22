use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) enum DestructureItem {
    Variable(DestructureVariable),
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
    pub(crate) fn parse_until<C: StopCondition>(input: ParseStream) -> Result<Self> {
        Ok(match detect_preinterpret_grammar(input.cursor()) {
            PeekMatch::GroupedCommand => {
                return input
                    .span()
                    .err("Grouped commands are not currently supported in destructuring positions")
            }
            PeekMatch::FlattenedCommand => {
                return input.span().err(
                    "Flattened commands are not currently supported in destructuring positions",
                )
            }
            PeekMatch::GroupedVariable
            | PeekMatch::FlattenedVariable
            | PeekMatch::AppendVariableDestructuring => {
                Self::Variable(DestructureVariable::parse_until::<C>(input)?)
            }
            PeekMatch::Group(_) => Self::ExactGroup(input.parse()?),
            PeekMatch::NamedDestructuring => todo!(),
            PeekMatch::Other => match input.parse::<TokenTree>()? {
                TokenTree::Group(_) => {
                    unreachable!("Should have been already handled by InterpretationGroup above")
                }
                TokenTree::Punct(punct) => Self::ExactPunct(punct),
                TokenTree::Ident(ident) => Self::ExactIdent(ident),
                TokenTree::Literal(literal) => Self::ExactLiteral(literal),
            },
        })
    }
}

impl HandleDestructure for DestructureItem {
    fn handle_destructure(&self, input: ParseStream, interpreter: &mut Interpreter) -> Result<()> {
        match self {
            DestructureItem::Variable(variable) => {
                variable.handle_destructure(input, interpreter)?;
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
