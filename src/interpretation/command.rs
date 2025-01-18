use crate::internal_prelude::*;

pub(crate) enum CommandOutput {
    None,
    Literal(Literal),
    Ident(Ident),
    FlattenedStream(InterpretedStream),
    Grouped(InterpretedStream, Span),
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub(crate) enum CommandOutputKind {
    None,
    /// LiteralOrBool
    Value,
    Ident,
    FlattenedStream,
    GroupedStream(Span),
}

pub(crate) trait OutputKind {
    type Output;
    fn resolve(span: &DelimSpan, flattening: Option<Token![..]>) -> Result<CommandOutputKind>;
}

pub(crate) struct OutputKindNone;
impl OutputKind for OutputKindNone {
    type Output = ();

    fn resolve(_: &DelimSpan, flattening: Option<Token![..]>) -> Result<CommandOutputKind> {
        match flattening {
            Some(dots) => dots.err("This command has no output, so cannot be flattened with .."),
            None => Ok(CommandOutputKind::None),
        }
    }
}

pub(crate) struct OutputKindValue;
impl OutputKind for OutputKindValue {
    type Output = TokenTree;

    fn resolve(_: &DelimSpan, flattening: Option<Token![..]>) -> Result<CommandOutputKind> {
        match flattening {
            Some(dots) => {
                dots.err("This command outputs a single value, so cannot be flattened with ..")
            }
            None => Ok(CommandOutputKind::Value),
        }
    }
}

pub(crate) struct OutputKindIdent;
impl OutputKind for OutputKindIdent {
    type Output = Ident;

    fn resolve(_: &DelimSpan, flattening: Option<Token![..]>) -> Result<CommandOutputKind> {
        match flattening {
            Some(dots) => {
                dots.err("This command outputs a single ident, so cannot be flattened with ..")
            }
            None => Ok(CommandOutputKind::Ident),
        }
    }
}

pub(crate) struct OutputKindStreaming;
impl OutputKind for OutputKindStreaming {
    type Output = InterpretedStream;

    fn resolve(span: &DelimSpan, flattening: Option<Token![..]>) -> Result<CommandOutputKind> {
        match flattening {
            Some(_) => Ok(CommandOutputKind::FlattenedStream),
            None => Ok(CommandOutputKind::GroupedStream(span.join())),
        }
    }
}

pub(crate) trait CommandType {
    type OutputKind: OutputKind;
}

pub(crate) trait NoOutputCommandDefinition:
    Sized + CommandType<OutputKind = OutputKindNone>
{
    const COMMAND_NAME: &'static str;
    fn parse(arguments: CommandArguments) -> Result<Self>;
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<()>;
}

impl<C: NoOutputCommandDefinition> CommandInvocationAs<OutputKindNone> for C {
    fn execute_into(
        self: Box<Self>,
        _: CommandOutputKind,
        interpreter: &mut Interpreter,
        _: &mut InterpretedStream,
    ) -> Result<()> {
        self.execute(interpreter)?;
        Ok(())
    }

    fn execute_into_value(
        self: Box<Self>,
        _: CommandOutputKind,
        interpreter: &mut Interpreter,
    ) -> Result<CommandOutput> {
        self.execute(interpreter)?;
        Ok(CommandOutput::None)
    }
}

pub(crate) trait IdentCommandDefinition:
    Sized + CommandType<OutputKind = OutputKindIdent>
{
    const COMMAND_NAME: &'static str;
    fn parse(arguments: CommandArguments) -> Result<Self>;
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<Ident>;
}

impl<C: IdentCommandDefinition> CommandInvocationAs<OutputKindIdent> for C {
    fn execute_into(
        self: Box<Self>,
        _: CommandOutputKind,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        output.push_ident(self.execute(interpreter)?);
        Ok(())
    }

    fn execute_into_value(
        self: Box<Self>,
        _: CommandOutputKind,
        interpreter: &mut Interpreter,
    ) -> Result<CommandOutput> {
        Ok(CommandOutput::Ident(self.execute(interpreter)?))
    }
}

pub(crate) trait ValueCommandDefinition:
    Sized + CommandType<OutputKind = OutputKindValue>
{
    const COMMAND_NAME: &'static str;
    fn parse(arguments: CommandArguments) -> Result<Self>;
    fn execute(self: Box<Self>, interpreter: &mut Interpreter) -> Result<TokenTree>;
}

impl<C: ValueCommandDefinition> CommandInvocationAs<OutputKindValue> for C {
    fn execute_into(
        self: Box<Self>,
        _: CommandOutputKind,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        output.push_raw_token_tree(self.execute(interpreter)?);
        Ok(())
    }

    fn execute_into_value(
        self: Box<Self>,
        _: CommandOutputKind,
        interpreter: &mut Interpreter,
    ) -> Result<CommandOutput> {
        Ok(match self.execute(interpreter)? {
            TokenTree::Literal(literal) => CommandOutput::Literal(literal),
            TokenTree::Ident(ident) => CommandOutput::Ident(ident),
            _ => panic!("Value Output Commands should only output literals or idents"),
        })
    }
}

pub(crate) trait StreamingCommandDefinition:
    Sized + CommandType<OutputKind = OutputKindStreaming>
{
    const COMMAND_NAME: &'static str;
    fn parse(arguments: CommandArguments) -> Result<Self>;
    fn execute(
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()>;
}

impl<C: StreamingCommandDefinition> CommandInvocationAs<OutputKindStreaming> for C {
    fn execute_into(
        self: Box<Self>,
        output_kind: CommandOutputKind,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        match output_kind {
            CommandOutputKind::FlattenedStream => self.execute(interpreter, output),
            CommandOutputKind::GroupedStream(span) => output.push_grouped(
                |inner| self.execute(interpreter, inner),
                Delimiter::None,
                span,
            ),
            _ => unreachable!(),
        }
    }

    fn execute_into_value(
        self: Box<Self>,
        output_kind: CommandOutputKind,
        interpreter: &mut Interpreter,
    ) -> Result<CommandOutput> {
        let mut output = InterpretedStream::new(SpanRange::ignored());
        self.execute(interpreter, &mut output)?;
        Ok(match output_kind {
            CommandOutputKind::FlattenedStream => CommandOutput::FlattenedStream(output),
            CommandOutputKind::GroupedStream(span) => CommandOutput::Grouped(output, span),
            _ => unreachable!(),
        })
    }
}

//=========================

// Using the trick for permitting multiple non-overlapping blanket
// implementations, conditioned on an associated type
pub(crate) trait CommandInvocationAs<T: OutputKind> {
    fn execute_into(
        self: Box<Self>,
        output_kind: CommandOutputKind,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()>;

    fn execute_into_value(
        self: Box<Self>,
        output_kind: CommandOutputKind,
        interpreter: &mut Interpreter,
    ) -> Result<CommandOutput>;
}

impl<C: CommandType + CommandInvocationAs<C::OutputKind>> CommandInvocation for C {
    fn execute_into(
        self: Box<Self>,
        output_kind: CommandOutputKind,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        <Self as CommandInvocationAs<C::OutputKind>>::execute_into(
            self,
            output_kind,
            interpreter,
            output,
        )
    }

    fn execute_into_value(
        self: Box<Self>,
        output_kind: CommandOutputKind,
        interpreter: &mut Interpreter,
    ) -> Result<CommandOutput> {
        <Self as CommandInvocationAs<C::OutputKind>>::execute_into_value(
            self,
            output_kind,
            interpreter,
        )
    }
}

pub(crate) trait CommandInvocation {
    fn execute_into(
        self: Box<Self>,
        output_kind: CommandOutputKind,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()>;

    fn execute_into_value(
        self: Box<Self>,
        output_kind: CommandOutputKind,
        interpreter: &mut Interpreter,
    ) -> Result<CommandOutput>;
}

pub(crate) trait ClonableCommandInvocation: CommandInvocation {
    fn clone_box(&self) -> Box<dyn ClonableCommandInvocation>;
}

impl<C: Clone + CommandInvocation + 'static> ClonableCommandInvocation for C {
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
                            $command::parse(arguments)?
                        ),
                    )*
                })
            }

            pub(crate) fn output_kind(&self, span: &DelimSpan, flattening: Option<Token![..]>) -> Result<CommandOutputKind> {
                match self {
                    $(
                        Self::$command => <$command as CommandType>::OutputKind::resolve(span, flattening),
                    )*
                }
            }

            pub(crate) fn for_ident(ident: &Ident) -> Option<Self> {
                Some(match ident.to_string().as_ref() {
                    $(
                        $command::COMMAND_NAME => Self::$command,
                    )*
                    _ => return None,
                })
            }

            const ALL_KIND_NAMES: &'static [&'static str] = &[$($command::COMMAND_NAME,)*];

            pub(crate) fn list_all() -> String {
                // TODO improve to add an "and" at the end
                Self::ALL_KIND_NAMES.join(", ")
            }
        }
    };
}
pub(crate) use define_command_kind;

#[derive(Clone)]
pub(crate) struct Command {
    invocation: Box<dyn ClonableCommandInvocation>,
    output_kind: CommandOutputKind,
    source_group_span: DelimSpan,
}

impl Parse for Command {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let open_bracket = syn::bracketed!(content in input);
        content.parse::<Token![!]>()?;
        let flattening = if content.peek(Token![.]) {
            Some(content.parse::<Token![..]>()?)
        } else {
            None
        };
        let command_name = content.call(Ident::parse_any)?;
        let (command_kind, output_kind) = match CommandKind::for_ident(&command_name) {
            Some(command_kind) => {
                let output_kind = command_kind.output_kind(&open_bracket.span, flattening)?;
                (command_kind, output_kind)
            }
            None => command_name.span().err(
                format!(
                    "Expected `[!<command>! ..]`, for <command> one of: {}.\nIf this wasn't intended to be a preinterpret command, you can work around this with [!raw! [!{} ... ]]",
                    CommandKind::list_all(),
                    command_name,
                ),
            )?,
        };
        content.parse::<Token![!]>()?;
        let invocation = command_kind.parse_invocation(CommandArguments::new(
            &content,
            command_name,
            open_bracket.span.span_range(),
        ))?;
        Ok(Self {
            invocation,
            output_kind,
            source_group_span: open_bracket.span,
        })
    }
}

impl Command {
    pub(crate) fn output_kind(&self) -> CommandOutputKind {
        self.output_kind
    }

    /// Should only be used to swap valid kinds
    pub(crate) unsafe fn set_output_kind(&mut self, output_kind: CommandOutputKind) {
        self.output_kind = output_kind;
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
        self.invocation
            .execute_into(self.output_kind, interpreter, output)
    }
}

impl Express for Command {
    fn add_to_expression(
        self,
        interpreter: &mut Interpreter,
        expression_stream: &mut ExpressionBuilder,
    ) -> Result<()> {
        match self
            .invocation
            .execute_into_value(self.output_kind, interpreter)?
        {
            CommandOutput::None => {}
            CommandOutput::Literal(literal) => {
                expression_stream.push_literal(literal);
            }
            CommandOutput::Ident(ident) => {
                expression_stream.push_ident(ident);
            }
            CommandOutput::Grouped(stream, span) => {
                expression_stream.push_grouped_interpreted_stream(stream, span);
            }
            CommandOutput::FlattenedStream(stream) => {
                expression_stream
                    .push_grouped_interpreted_stream(stream, self.source_group_span.join());
            }
        };
        Ok(())
    }
}
