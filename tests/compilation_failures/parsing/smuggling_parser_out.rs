use preinterpret::*;

fn main() {
    let _ = run! {
        let smuggled_input;
        parse %[Hello World] => |input| {
            let _ = input.ident();
            let _ = input.ident();
            smuggled_input = input;
        }
        let _ = smuggled_input.ident();
    };
}