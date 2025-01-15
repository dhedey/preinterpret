use crate::internal_prelude::*;

pub(crate) trait CommandDefinition: CommandInvocation + Clone {
    const COMMAND_NAME: &'static str;

    fn parse(arguments: CommandArguments) -> Result<Self>;
}

pub(crate) trait CommandInvocation {
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<CommandOutput>;
}

pub(crate) enum CommandOutput {
    Empty,
    Literal(Literal),
    Ident(Ident),
    AppendStream(InterpretedStream),
    GroupedStream(InterpretedStream),
}

pub(crate) trait ClonableCommandInvocation: CommandInvocation {
    fn clone_box(&self) -> Box<dyn ClonableCommandInvocation>;
}

impl<C: CommandDefinition + CommandInvocation + 'static> ClonableCommandInvocation for C {
    fn clone_box(&self) -> Box<dyn ClonableCommandInvocation> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ClonableCommandInvocation> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

macro_rules! define_command_kind {
    (
        $(
            $command:ident,
        )*
    ) => {
        #[allow(clippy::enum_variant_names)]
        #[derive(Clone, Copy)]
        pub(crate) enum CommandKind {
            $(
                $command,
            )*
        }

        impl CommandKind {
            pub(crate) fn parse_invocation(&self, arguments: CommandArguments) -> Result<Box<dyn ClonableCommandInvocation>> {
                Ok(match self {
                    $(
                        Self::$command => Box::new(
                            <$command as CommandDefinition>::parse(arguments)?
                        ),
                    )*
                })
            }

            pub(crate) fn for_ident(ident: &Ident) -> Option<Self> {
                Some(match ident.to_string().as_ref() {
                    $(
                        <$command as CommandDefinition>::COMMAND_NAME => Self::$command,
                    )*
                    _ => return None,
                })
            }

            const ALL_KIND_NAMES: &'static [&'static str] = &[$(<$command as CommandDefinition>::COMMAND_NAME,)*];

            pub(crate) fn list_all() -> String {
                // TODO improve to add an "and" at the end
                Self::ALL_KIND_NAMES.join(", ")
            }
        }
    };
}
pub(crate) use define_command_kind;

impl Parse for CommandKind {
    fn parse(input: ParseStream) -> Result<Self> {
        // Support parsing any ident
        let ident = input.call(Ident::parse_any)?;
        match Self::for_ident(&ident) {
            Some(command_kind) => Ok(command_kind),
            None => ident.span().err(
                format!(
                    "Expected `[!<command>! ..]`, for <command> one of: {}.\nIf this wasn't intended to be a preinterpret command, you can work around this with [!raw! [!{} ... ]]",
                    Self::list_all(),
                    ident,
                ),
            ),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Command {
    invocation: Box<dyn ClonableCommandInvocation>,
    source_group_span: DelimSpan,
}

impl Parse for Command {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let open_bracket = syn::bracketed!(content in input);
        content.parse::<Token![!]>()?;
        let command_kind = content.parse::<CommandKind>()?;
        content.parse::<Token![!]>()?;
        let invocation = command_kind.parse_invocation(CommandArguments::new(
            &content,
            open_bracket.span.span_range(),
        ))?;
        Ok(Self {
            invocation,
            source_group_span: open_bracket.span,
        })
    }
}

impl HasSpanRange for Command {
    fn span_range(&self) -> SpanRange {
        self.source_group_span.span_range()
    }
}

impl Interpret for Command {
    fn interpret_as_tokens_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        match self.invocation.execute(interpreter)? {
            CommandOutput::Empty => {}
            CommandOutput::Literal(literal) => {
                output.push_literal(literal);
            }
            CommandOutput::Ident(ident) => {
                output.push_ident(ident);
            }
            CommandOutput::AppendStream(stream) => {
                output.extend(stream);
            }
            CommandOutput::GroupedStream(stream) => {
                output.push_new_group(stream, Delimiter::None, self.source_group_span.join());
            }
        };
        Ok(())
    }

    fn interpret_as_expression_into(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionStream,
    ) -> Result<()> {
        match self.invocation.execute(interpreter)? {
            CommandOutput::Empty => {}
            CommandOutput::Literal(literal) => {
                expression_stream.push_literal(literal);
            }
            CommandOutput::Ident(ident) => {
                expression_stream.push_ident(ident);
            }
            CommandOutput::AppendStream(stream) | CommandOutput::GroupedStream(stream) => {
                expression_stream
                    .push_grouped_interpreted_stream(stream, self.source_group_span.join());
            }
        };
        Ok(())
    }
}
