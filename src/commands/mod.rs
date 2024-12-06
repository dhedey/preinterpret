mod concat_commands;
mod core_commands;

use crate::internal_prelude::*;
use concat_commands::*;
use core_commands::*;

define_commands! {
    pub(crate) enum CommandKind {
        // Core Commands
        SetCommand,
        RawCommand,
        IgnoreCommand,

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
    }
}
