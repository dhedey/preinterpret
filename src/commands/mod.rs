mod boolean_commands;
mod core_commands;
mod concat_commands;
mod control_flow_commands;
mod integer_commands;

use crate::internal_prelude::*;
use boolean_commands::*;
use core_commands::*;
use concat_commands::*;
use control_flow_commands::*;
use integer_commands::*;

define_commands! {
    pub(crate) enum CommandKind {
        // Core Commands
        SetCommand,
        AsRawTokensCommand,
        IgnoreCommand,

        // Concat & Type Convert Commands
        ToStringCommand,
        ToIdentCommand,
        ToLiteralCommand,

        // Concat & String Convert Commands
        UpperCaseCommand,
        LowerCaseCommand,
        SnakeCaseCommand,
        LowerSnakeCaseCommand,
        UpperSnakeCaseCommand,
        CamelCaseCommand,
        LowerCamelCaseCommand,
        UpperCamelCaseCommand,
        CapitalizeCommand,
        DecapitalizeCommand,

        // Integer Commands

        // Boolean Commands

        // Control Flow Commands
    }
}