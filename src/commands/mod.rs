mod concat_commands;
mod control_flow_commands;
mod core_commands;
mod expression_commands;
mod token_commands;

use crate::internal_prelude::*;
use concat_commands::*;
use control_flow_commands::*;
use core_commands::*;
use expression_commands::*;
use token_commands::*;

define_command_kind! {
    // Core Commands
    SetCommand,
    ExtendCommand,
    RawCommand,
    IgnoreCommand,
    StreamCommand,
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

    // Control flow commands
    IfCommand,
    WhileCommand,

    // Token Commands
    EmptyCommand,
    IsEmptyCommand,
    LengthCommand,
    GroupCommand,
}
