use preinterpret::*;

fn main() {
    let _ = run!{
        0u8 as nonexistent_type
    };
}