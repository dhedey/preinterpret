use preinterpret::*;

// Float division by zero would produce infinity, which is not supported in preinterpret.
// This causes a compile-time panic because the UntypedFloat type requires finite values.
fn main() {
    let _ = run!(1.0f32 / 0.0f32);
}
