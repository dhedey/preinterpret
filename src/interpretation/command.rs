use super::commands::*;
use crate::internal_prelude::*;

#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommandOutputKind {
    None,
    FlattenedValue,
    GroupedValue,
    Ident,
    Literal,
    FlattenedStream,
    GroupedStream,
    Stream,
}

pub(crate) trait CommandType {
    type OutputKind: OutputKind;
}

pub(crate) trait OutputKind {
    type Output;
    fn resolve_standard() -> CommandOutputKind;
    fn resolve_flattened(error_span_range: SpanRange) -> ParseResult<CommandOutputKind>;
}

struct ExecutionContext<'a> {
    interpreter: &'a mut Interpreter,
    output_kind: CommandOutputKind,
    delim_span: DelimSpan,
}

trait CommandInvocation {
    fn execute_into(
        self,
        context: ExecutionContext,
        output: &mut OutputStream,
    ) -> ExecutionResult<()>;

    fn execute_to_value(self, context: ExecutionContext) -> ExecutionResult<ExpressionValue>;
}

// Using the trick for permitting multiple non-overlapping blanket
// implementations, conditioned on an associated type
trait CommandInvocationAs<T: OutputKind> {
    fn execute_into(
        self,
        context: ExecutionContext,
        output: &mut OutputStream,
    ) -> ExecutionResult<()>;

    fn execute_to_value(self, context: ExecutionContext) -> ExecutionResult<ExpressionValue>;
}

impl<C: CommandType + CommandInvocationAs<C::OutputKind>> CommandInvocation for C {
    fn execute_into(
        self,
        context: ExecutionContext,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        <Self as CommandInvocationAs<C::OutputKind>>::execute_into(self, context, output)
    }

    fn execute_to_value(self, context: ExecutionContext) -> ExecutionResult<ExpressionValue> {
        <Self as CommandInvocationAs<C::OutputKind>>::execute_to_value(self, context)
    }
}

//===============
// OutputKindNone
//===============

pub(crate) struct OutputKindNone;
impl OutputKind for OutputKindNone {
    type Output = ();

    fn resolve_standard() -> CommandOutputKind {
        CommandOutputKind::None
    }

    fn resolve_flattened(error_span_range: SpanRange) -> ParseResult<CommandOutputKind> {
        error_span_range.parse_err("This command has no output, so cannot be flattened with ..")
    }
}

pub(crate) trait NoOutputCommandDefinition:
    Sized + CommandType<OutputKind = OutputKindNone>
{
    const COMMAND_NAME: &'static str;
    fn parse(arguments: CommandArguments) -> ParseResult<Self>;
    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<()>;
}

impl<C: NoOutputCommandDefinition> CommandInvocationAs<OutputKindNone> for C {
    fn execute_into(self, context: ExecutionContext, _: &mut OutputStream) -> ExecutionResult<()> {
        self.execute(context.interpreter)?;
        Ok(())
    }

    fn execute_to_value(self, context: ExecutionContext) -> ExecutionResult<ExpressionValue> {
        self.execute(context.interpreter)?;
        Ok(ExpressionValue::None(context.delim_span.span_range()))
    }
}

//================
// OutputKindValue
//================

pub(crate) struct OutputKindValue;
impl OutputKind for OutputKindValue {
    type Output = TokenTree;

    fn resolve_standard() -> CommandOutputKind {
        CommandOutputKind::FlattenedValue
    }

    fn resolve_flattened(error_span_range: SpanRange) -> ParseResult<CommandOutputKind> {
        error_span_range
            .parse_err("This command outputs a single value, so cannot be flattened with ..")
    }
}

pub(crate) trait ValueCommandDefinition:
    Sized + CommandType<OutputKind = OutputKindValue>
{
    const COMMAND_NAME: &'static str;
    fn parse(arguments: CommandArguments) -> ParseResult<Self>;
    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<ExpressionValue>;
}

impl<C: ValueCommandDefinition> CommandInvocationAs<OutputKindValue> for C {
    fn execute_into(
        self,
        context: ExecutionContext,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let grouping = match context.output_kind {
            CommandOutputKind::FlattenedValue => Grouping::Flattened,
            _ => Grouping::Grouped,
        };
        self.execute(context.interpreter)?
            .output_to(grouping, output);
        Ok(())
    }

    fn execute_to_value(self, context: ExecutionContext) -> ExecutionResult<ExpressionValue> {
        self.execute(context.interpreter)
    }
}

//================
// OutputKindIdent
//================

pub(crate) struct OutputKindIdent;
impl OutputKind for OutputKindIdent {
    type Output = Ident;

    fn resolve_standard() -> CommandOutputKind {
        CommandOutputKind::Ident
    }

    fn resolve_flattened(error_span_range: SpanRange) -> ParseResult<CommandOutputKind> {
        error_span_range
            .parse_err("This command outputs a single ident, so cannot be flattened with ..")
    }
}

pub(crate) trait IdentCommandDefinition:
    Sized + CommandType<OutputKind = OutputKindIdent>
{
    const COMMAND_NAME: &'static str;
    fn parse(arguments: CommandArguments) -> ParseResult<Self>;
    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<Ident>;
}

impl<C: IdentCommandDefinition> CommandInvocationAs<OutputKindIdent> for C {
    fn execute_into(
        self,
        context: ExecutionContext,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        output.push_ident(self.execute(context.interpreter)?);
        Ok(())
    }

    fn execute_to_value(self, context: ExecutionContext) -> ExecutionResult<ExpressionValue> {
        let span_range = context.delim_span.span_range();
        let mut output = OutputStream::new();
        output.push_ident(self.execute(context.interpreter)?);
        Ok(output.to_value(span_range))
    }
}

//==================
// OutputKindLiteral
//==================

pub(crate) struct OutputKindLiteral;
impl OutputKind for OutputKindLiteral {
    type Output = Literal;

    fn resolve_standard() -> CommandOutputKind {
        CommandOutputKind::Literal
    }

    fn resolve_flattened(error_span_range: SpanRange) -> ParseResult<CommandOutputKind> {
        error_span_range
            .parse_err("This command outputs a single literal, so cannot be flattened with ..")
    }
}

pub(crate) trait LiteralCommandDefinition:
    Sized + CommandType<OutputKind = OutputKindLiteral>
{
    const COMMAND_NAME: &'static str;
    fn parse(arguments: CommandArguments) -> ParseResult<Self>;
    fn execute(self, interpreter: &mut Interpreter) -> ExecutionResult<Literal>;
}

impl<C: LiteralCommandDefinition> CommandInvocationAs<OutputKindLiteral> for C {
    fn execute_into(
        self,
        context: ExecutionContext,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        output.push_literal(self.execute(context.interpreter)?);
        Ok(())
    }

    fn execute_to_value(self, context: ExecutionContext) -> ExecutionResult<ExpressionValue> {
        let literal = self.execute(context.interpreter)?;
        Ok(ExpressionValue::for_literal(literal))
    }
}

//=================
// OutputKindStream
//=================

pub(crate) struct OutputKindGroupedStream;
impl OutputKind for OutputKindGroupedStream {
    type Output = OutputStream;

    fn resolve_standard() -> CommandOutputKind {
        CommandOutputKind::GroupedStream
    }

    fn resolve_flattened(_: SpanRange) -> ParseResult<CommandOutputKind> {
        Ok(CommandOutputKind::FlattenedStream)
    }
}

pub(crate) trait GroupedStreamCommandDefinition:
    Sized + CommandType<OutputKind = OutputKindGroupedStream>
{
    const COMMAND_NAME: &'static str;
    fn parse(arguments: CommandArguments) -> ParseResult<Self>;
    fn execute(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()>;
}

impl<C: GroupedStreamCommandDefinition> CommandInvocationAs<OutputKindGroupedStream> for C {
    fn execute_into(
        self,
        context: ExecutionContext,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        match context.output_kind {
            CommandOutputKind::FlattenedStream => self.execute(context.interpreter, output),
            CommandOutputKind::GroupedStream => output.push_grouped(
                |inner| self.execute(context.interpreter, inner),
                Delimiter::None,
                context.delim_span.join(),
            ),
            _ => unreachable!(),
        }
    }

    fn execute_to_value(self, context: ExecutionContext) -> ExecutionResult<ExpressionValue> {
        let span_range = context.delim_span.span_range();
        let mut output = OutputStream::new();
        <Self as CommandInvocationAs<OutputKindGroupedStream>>::execute_into(
            self,
            context,
            &mut output,
        )?;
        Ok(output.to_value(span_range))
    }
}

//======================
// OutputKindControlFlow
//======================

pub(crate) struct OutputKindStreaming;
impl OutputKind for OutputKindStreaming {
    type Output = ();

    fn resolve_standard() -> CommandOutputKind {
        CommandOutputKind::Stream
    }

    fn resolve_flattened(error_span_range: SpanRange) -> ParseResult<CommandOutputKind> {
        error_span_range.parse_err("This command always outputs a flattened stream and so cannot be explicitly flattened. If it needs to be grouped, wrap it in a [!group! ..] command")
    }
}

// Control Flow or a command which is unlikely to want grouped output
pub(crate) trait StreamingCommandDefinition:
    Sized + CommandType<OutputKind = OutputKindStreaming>
{
    const COMMAND_NAME: &'static str;
    fn parse(arguments: CommandArguments) -> ParseResult<Self>;
    fn execute(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()>;
}

impl<C: StreamingCommandDefinition> CommandInvocationAs<OutputKindStreaming> for C {
    fn execute_into(
        self,
        context: ExecutionContext,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        self.execute(context.interpreter, output)
    }

    fn execute_to_value(self, context: ExecutionContext) -> ExecutionResult<ExpressionValue> {
        let span_range = context.delim_span.span_range();
        let mut output = OutputStream::new();
        <Self as CommandInvocationAs<OutputKindStreaming>>::execute_into(
            self,
            context,
            &mut output,
        )?;
        Ok(output.to_value(span_range))
    }
}

//=========================

macro_rules! define_command_enums {
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
            fn parse_command(&self, arguments: CommandArguments) -> ParseResult<TypedCommand> {
                Ok(match self {
                    $(
                        Self::$command => TypedCommand::$command(
                            $command::parse(arguments)?
                        ),
                    )*
                })
            }

            pub(crate) fn standard_output_kind(&self) -> CommandOutputKind {
                match self {
                    $(
                        Self::$command => <$command as CommandType>::OutputKind::resolve_standard(),
                    )*
                }
            }

            pub(crate) fn flattened_output_kind(&self, error_span_range: SpanRange) -> ParseResult<CommandOutputKind> {
                match self {
                    $(
                        Self::$command => <$command as CommandType>::OutputKind::resolve_flattened(error_span_range),
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

        #[allow(clippy::enum_variant_names)]
        #[derive(Clone)]
        enum TypedCommand {
            $(
                $command($command),
            )*
        }

        impl TypedCommand {
            fn execute_into(
                self,
                context: ExecutionContext,
                output: &mut OutputStream,
            ) -> ExecutionResult<()> {
                match self {
                    $(
                        Self::$command(command) => <$command as CommandInvocation>::execute_into(command, context, output),
                    )*
                }
            }

            fn execute_to_value(self, context: ExecutionContext) -> ExecutionResult<ExpressionValue> {
                match self {
                    $(
                        Self::$command(command) => <$command as CommandInvocation>::execute_to_value(command, context),
                    )*
                }
            }
        }
    };
}

define_command_enums! {
    // Core Commands
    SetCommand,
    TypedSetCommand,
    RawCommand,
    OutputCommand,
    IgnoreCommand,
    ReinterpretCommand,
    SettingsCommand,
    ErrorCommand,
    DebugCommand,

    // Concat & Type Convert Commands
    StringCommand,
    IdentCommand,
    IdentCamelCommand,
    IdentSnakeCommand,
    IdentUpperSnakeCommand,
    LiteralCommand,

    // Concat & String Convert Commands
    UpperCommand,
    LowerCommand,
    SnakeCommand,
    LowerSnakeCommand,
    UpperSnakeCommand,
    CamelCommand,
    LowerCamelCommand,
    UpperCamelCommand,
    KebabCommand,
    CapitalizeCommand,
    DecapitalizeCommand,
    TitleCommand,
    InsertSpacesCommand,

    // Expression Commands
    RangeCommand,

    // Control flow commands
    IfCommand,
    WhileCommand,
    ForCommand,
    LoopCommand,
    ContinueCommand,
    BreakCommand,

    // Token Commands
    IsEmptyCommand,
    LengthCommand,
    GroupCommand,
    IntersperseCommand,
    SplitCommand,
    CommaSplitCommand,
    ZipCommand,

    // Destructuring Commands
    ParseCommand,
    LetCommand,
}

#[derive(Clone)]
pub(crate) struct Command {
    typed: Box<TypedCommand>,
    output_kind: CommandOutputKind,
    source_group_span: DelimSpan,
}

impl Parse<Source> for Command {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let (delim_span, content) = input.parse_specific_group(Delimiter::Bracket)?;
        content.parse::<Token![!]>()?;
        let flattening = if content.peek(Token![.]) {
            Some(content.parse::<Token![..]>()?)
        } else {
            None
        };
        let command_name = content.parse_any_ident()?;
        let (command_kind, output_kind) = match CommandKind::for_ident(&command_name) {
            Some(command_kind) => {
                let output_kind = match flattening {
                    Some(flattening) => command_kind.flattened_output_kind(flattening.span_range())?,
                    None => command_kind.standard_output_kind(),
                };
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
        let typed = command_kind.parse_command(CommandArguments::new(
            &content,
            command_name,
            delim_span.join(),
        ))?;
        Ok(Self {
            typed: Box::new(typed),
            output_kind,
            source_group_span: delim_span,
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

impl HasSpan for Command {
    fn span(&self) -> Span {
        self.source_group_span.join()
    }
}

impl Interpret for Command {
    fn interpret_into(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()> {
        let context = ExecutionContext {
            interpreter,
            output_kind: self.output_kind,
            delim_span: self.source_group_span,
        };
        self.typed.execute_into(context, output)
    }
}

impl InterpretToValue for Command {
    type OutputValue = ExpressionValue;

    fn interpret_to_value(
        self,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<Self::OutputValue> {
        let context = ExecutionContext {
            interpreter,
            output_kind: self.output_kind,
            delim_span: self.source_group_span,
        };
        self.typed.execute_to_value(context)
    }
}
