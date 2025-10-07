use preinterpret::*;

fn main() {
    let _ = run!{
        x = %[+ 1];
        1 x
    };
}