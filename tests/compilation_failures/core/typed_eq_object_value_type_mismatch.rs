use preinterpret::*;

fn main() {
    run!(
        let result = %{ a: 1, b: 2 }.typed_eq(%{ a: 1, b: "two" });
    );
}
