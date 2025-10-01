use super::commands::*;
use crate::internal_prelude::*;

#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommandOutputKind {
    None,
    /// If output to a parent stream, it is flattened
    Value,
    Ident,
    /// If output to a parent stream, it is flattened
    Literal,
    /// If output to a parent stream, it is flattened
    Stream,
}

pub(crate) trait CommandType {
    type OutputKind: OutputKind;
}

pub(crate) trait OutputKind {
    type Output;
    fn resolve_enum_kind() -> CommandOutputKind;
}

struct ExecutionContext<'a> {
    interpreter: &'a mut Interpreter,
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

    fn resolve_enum_kind() -> CommandOutputKind {
        CommandOutputKind::None
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

    fn resolve_enum_kind() -> CommandOutputKind {
        CommandOutputKind::Value
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
        self.execute(context.interpreter)?
            .output_to(Grouping::Grouped, output)
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

    fn resolve_enum_kind() -> CommandOutputKind {
        CommandOutputKind::Ident
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

    fn resolve_enum_kind() -> CommandOutputKind {
        CommandOutputKind::Literal
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

pub(crate) struct OutputKindStream;
impl OutputKind for OutputKindStream {
    type Output = ();

    fn resolve_enum_kind() -> CommandOutputKind {
        CommandOutputKind::Stream
    }
}

// Control Flow or a command which is unlikely to want grouped output
pub(crate) trait StreamCommandDefinition:
    Sized + CommandType<OutputKind = OutputKindStream>
{
    const COMMAND_NAME: &'static str;
    fn parse(arguments: CommandArguments) -> ParseResult<Self>;
    fn execute(
        self,
        interpreter: &mut Interpreter,
        output: &mut OutputStream,
    ) -> ExecutionResult<()>;
}

impl<C: StreamCommandDefinition> CommandInvocationAs<OutputKindStream> for C {
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
        <Self as CommandInvocationAs<OutputKindStream>>::execute_into(self, context, &mut output)?;
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

            pub(crate) fn resolve_output_kind(&self) -> CommandOutputKind {
                match self {
                    $(
                        Self::$command => <$command as CommandType>::OutputKind::resolve_enum_kind(),
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
                // TODO: Separate by group, and add "and" at the end
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
    ReinterpretCommand,
    SettingsCommand,
    ErrorCommand,

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
    IntersperseCommand,
    SplitCommand,
    CommaSplitCommand,
    ZipCommand,
    ZipTruncatedCommand,

    // Destructuring Commands
    ParseCommand,
}

#[derive(Clone)]
pub(crate) struct Command {
    typed: Box<TypedCommand>,
    brackets: Brackets,
}

impl Parse<Source> for Command {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let (brackets, content) = input.parse_brackets()?;
        content.parse::<Token![!]>()?;
        let command_name = content.parse_any_ident()?;
        let command_kind = match CommandKind::for_ident(&command_name) {
            Some(command_kind) => command_kind,
            None => command_name.span().err(
                format!(
                    "Expected `[!<command>! ..]`, for <command> one of: {}.\nIf this wasn't intended to be a preinterpret command, you can work around this with %raw[[!{} ... ]]",
                    CommandKind::list_all(),
                    command_name,
                ),
            )?,
        };
        content.parse::<Token![!]>()?;
        let typed = command_kind.parse_command(CommandArguments::new(
            &content,
            command_name,
            brackets.join(),
        ))?;
        Ok(Self {
            typed: Box::new(typed),
            brackets,
        })
    }
}

impl HasSpan for Command {
    fn span(&self) -> Span {
        self.brackets.join()
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
            delim_span: self.brackets.delim_span,
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
            delim_span: self.brackets.delim_span,
        };
        self.typed.execute_to_value(context)
    }
}
