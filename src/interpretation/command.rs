use crate::internal_prelude::*;

#[allow(unused)]
#[derive(Clone, Copy)]
pub(crate) enum CommandOutputKind {
    None,
    /// LiteralOrBool
    Value,
    Ident,
    FlattenedStream,
    GroupedStream,
    ControlFlowCodeStream,
}

pub(crate) trait CommandType {
    type OutputKind: OutputKind;
}

pub(crate) trait OutputKind {
    type Output;
    fn resolve(flattening: Option<Token![..]>) -> Result<CommandOutputKind>;
}

struct ExecutionContext<'a> {
    interpreter: &'a mut Interpreter,
    output_kind: CommandOutputKind,
    delim_span: DelimSpan,
}

trait CommandInvocation {
    fn execute_into(
        self: Box<Self>,
        context: ExecutionContext,
        output: &mut InterpretedStream,
    ) -> Result<()>;

    fn execute_into_expression(
        self: Box<Self>,
        context: ExecutionContext,
        builder: &mut ExpressionBuilder,
    ) -> Result<()>;
}

trait ClonableCommandInvocation: CommandInvocation {
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

// Using the trick for permitting multiple non-overlapping blanket
// implementations, conditioned on an associated type
trait CommandInvocationAs<T: OutputKind> {
    fn execute_into(
        self: Box<Self>,
        context: ExecutionContext,
        output: &mut InterpretedStream,
    ) -> Result<()>;

    fn execute_into_expression(
        self: Box<Self>,
        context: ExecutionContext,
        builder: &mut ExpressionBuilder,
    ) -> Result<()>;
}

impl<C: CommandType + CommandInvocationAs<C::OutputKind>> CommandInvocation for C {
    fn execute_into(
        self: Box<Self>,
        context: ExecutionContext,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        <Self as CommandInvocationAs<C::OutputKind>>::execute_into(self, context, output)
    }

    fn execute_into_expression(
        self: Box<Self>,
        context: ExecutionContext,
        builder: &mut ExpressionBuilder,
    ) -> Result<()> {
        <Self as CommandInvocationAs<C::OutputKind>>::execute_into_expression(
            self, context, builder,
        )
    }
}

//===============
// OutputKindNone
//===============

pub(crate) struct OutputKindNone;
impl OutputKind for OutputKindNone {
    type Output = ();

    fn resolve(flattening: Option<Token![..]>) -> Result<CommandOutputKind> {
        match flattening {
            Some(dots) => dots.err("This command has no output, so cannot be flattened with .."),
            None => Ok(CommandOutputKind::None),
        }
    }
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
        context: ExecutionContext,
        _: &mut InterpretedStream,
    ) -> Result<()> {
        self.execute(context.interpreter)?;
        Ok(())
    }

    fn execute_into_expression(
        self: Box<Self>,
        context: ExecutionContext,
        _: &mut ExpressionBuilder,
    ) -> Result<()> {
        context.delim_span
            .join()
            .err("Commands with no output cannot be used directly in expressions.\nConsider wrapping it inside a command such as [!group! ..] which returns an expression")
    }
}

//================
// OutputKindValue
//================

pub(crate) struct OutputKindValue;
impl OutputKind for OutputKindValue {
    type Output = TokenTree;

    fn resolve(flattening: Option<Token![..]>) -> Result<CommandOutputKind> {
        match flattening {
            Some(dots) => {
                dots.err("This command outputs a single value, so cannot be flattened with ..")
            }
            None => Ok(CommandOutputKind::Value),
        }
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
        context: ExecutionContext,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        output.push_raw_token_tree(self.execute(context.interpreter)?);
        Ok(())
    }

    fn execute_into_expression(
        self: Box<Self>,
        context: ExecutionContext,
        builder: &mut ExpressionBuilder,
    ) -> Result<()> {
        match self.execute(context.interpreter)? {
            TokenTree::Literal(literal) => builder.push_literal(literal),
            TokenTree::Ident(ident) => builder.push_ident(ident),
            _ => panic!("Value Output Commands should only output literals or idents"),
        }
        Ok(())
    }
}

//================
// OutputKindIdent
//================

pub(crate) struct OutputKindIdent;
impl OutputKind for OutputKindIdent {
    type Output = Ident;

    fn resolve(flattening: Option<Token![..]>) -> Result<CommandOutputKind> {
        match flattening {
            Some(dots) => {
                dots.err("This command outputs a single ident, so cannot be flattened with ..")
            }
            None => Ok(CommandOutputKind::Ident),
        }
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
        context: ExecutionContext,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        output.push_ident(self.execute(context.interpreter)?);
        Ok(())
    }

    fn execute_into_expression(
        self: Box<Self>,
        context: ExecutionContext,
        builder: &mut ExpressionBuilder,
    ) -> Result<()> {
        builder.push_ident(self.execute(context.interpreter)?);
        Ok(())
    }
}

//=================
// OutputKindStream
//=================

pub(crate) struct OutputKindStream;
impl OutputKind for OutputKindStream {
    type Output = InterpretedStream;

    fn resolve(flattening: Option<Token![..]>) -> Result<CommandOutputKind> {
        match flattening {
            Some(_) => Ok(CommandOutputKind::FlattenedStream),
            None => Ok(CommandOutputKind::GroupedStream),
        }
    }
}

pub(crate) trait StreamCommandDefinition:
    Sized + CommandType<OutputKind = OutputKindStream>
{
    const COMMAND_NAME: &'static str;
    fn parse(arguments: CommandArguments) -> Result<Self>;
    fn execute(
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()>;
}

impl<C: StreamCommandDefinition> CommandInvocationAs<OutputKindStream> for C {
    fn execute_into(
        self: Box<Self>,
        context: ExecutionContext,
        output: &mut InterpretedStream,
    ) -> Result<()> {
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

    fn execute_into_expression(
        self: Box<Self>,
        context: ExecutionContext,
        builder: &mut ExpressionBuilder,
    ) -> Result<()> {
        if let CommandOutputKind::FlattenedStream = context.output_kind {
            return context.delim_span
                .join()
                .err("Flattened commands cannot be used directly in expressions.\nConsider removing the .. or wrapping it inside a command such as [!group! ..] which returns an expression");
        }
        builder.push_grouped(
            |output| self.execute(context.interpreter, output),
            context.delim_span.join(),
        )
    }
}

//======================
// OutputKindControlFlow
//======================

pub(crate) struct OutputKindControlFlow;
impl OutputKind for OutputKindControlFlow {
    type Output = ();

    fn resolve(flattening: Option<Token![..]>) -> Result<CommandOutputKind> {
        match flattening {
            Some(dots) => dots.err("This command is control flow, so is always flattened and cannot be flattened. If it needs to be grouped, wrap it in a [!group! ..] command"),
            None => Ok(CommandOutputKind::ControlFlowCodeStream),
        }
    }
}

pub(crate) trait ControlFlowCommandDefinition:
    Sized + CommandType<OutputKind = OutputKindControlFlow>
{
    const COMMAND_NAME: &'static str;
    fn parse(arguments: CommandArguments) -> Result<Self>;
    fn execute(
        self: Box<Self>,
        interpreter: &mut Interpreter,
        output: &mut InterpretedStream,
    ) -> Result<()>;
}

impl<C: ControlFlowCommandDefinition> CommandInvocationAs<OutputKindControlFlow> for C {
    fn execute_into(
        self: Box<Self>,
        context: ExecutionContext,
        output: &mut InterpretedStream,
    ) -> Result<()> {
        self.execute(context.interpreter, output)
    }

    fn execute_into_expression(
        self: Box<Self>,
        context: ExecutionContext,
        builder: &mut ExpressionBuilder,
    ) -> Result<()> {
        builder.push_grouped(
            |output| self.execute(context.interpreter, output),
            context.delim_span.join(),
        )
    }
}

//=========================

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
            fn parse_invocation(&self, arguments: CommandArguments) -> Result<Box<dyn ClonableCommandInvocation>> {
                Ok(match self {
                    $(
                        Self::$command => Box::new(
                            $command::parse(arguments)?
                        ),
                    )*
                })
            }

            pub(crate) fn output_kind(&self, flattening: Option<Token![..]>) -> Result<CommandOutputKind> {
                match self {
                    $(
                        Self::$command => <$command as CommandType>::OutputKind::resolve(flattening),
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

define_command_kind! {
    // Core Commands
    SetCommand,
    ExtendCommand,
    RawCommand,
    IgnoreCommand,
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

    // Expression Commands
    EvaluateCommand,
    AssignCommand,
    RangeCommand,

    // Control flow commands
    IfCommand,
    WhileCommand,

    // Token Commands
    IsEmptyCommand,
    LengthCommand,
    GroupCommand,
    IntersperseCommand,
}

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
                let output_kind = command_kind.output_kind(flattening)?;
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
        let context = ExecutionContext {
            interpreter,
            output_kind: self.output_kind,
            delim_span: self.source_group_span,
        };
        self.invocation.execute_into(context, output)
    }
}

impl Express for Command {
    fn add_to_expression(
        self,
        interpreter: &mut Interpreter,
        builder: &mut ExpressionBuilder,
    ) -> Result<()> {
        let context = ExecutionContext {
            interpreter,
            output_kind: self.output_kind,
            delim_span: self.source_group_span,
        };
        // This is set up so that we can determine the exact expression
        // structure at parse time, and in future refactor to parsing
        // the expression ourselves, and then executing an expression AST
        // rather than going via a syn::expression
        self.invocation.execute_into_expression(context, builder)
    }
}
