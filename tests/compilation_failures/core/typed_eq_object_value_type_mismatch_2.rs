use preinterpret::*;

fn main() {
    run!(
        let result = %{ a: 1, ["multi-word key"]: 2 }.typed_eq(%{ a: 1, ["multi-word key"]: "two" });
    );
}
