use preinterpret::*;

fn main() {
    let _ = stream!{
        #("hello" * 3)
    };
}
