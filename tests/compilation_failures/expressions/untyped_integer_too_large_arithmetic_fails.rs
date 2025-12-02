use preinterpret::*;

fn main() {
    let _ = run!(
        // (i128::MAX + 1) so we're an unsupported literal
        let x = 170_141_183_460_469_231_731_687_303_715_884_105_728;
        // This should error because an unsupported literal doesn't support operations
        x - 1
    );
}