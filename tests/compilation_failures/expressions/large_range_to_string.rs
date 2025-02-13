use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        #([!range! 0..100000] as string)
    };
}