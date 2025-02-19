use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        #({ hello: "world" })
    };
}