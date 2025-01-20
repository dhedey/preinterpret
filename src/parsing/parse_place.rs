use crate::internal_prelude::*;

#[derive(Clone)]
pub(crate) enum ParsePlace {
    GroupedVariable(GroupedVariable),
}

impl Parse for ParsePlace {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self::GroupedVariable(input.parse()?))
    }
}

impl HandleParse for ParsePlace {
    fn handle_parse(&self, input: ParseStream, interpreter: &mut Interpreter) -> Result<()> {
        match self {
            Self::GroupedVariable(grouped_variable) => {
                grouped_variable.handle_parse(input, interpreter)
            }
        }
    }
}
