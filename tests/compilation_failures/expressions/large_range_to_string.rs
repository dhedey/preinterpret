use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        #((0..10000) as iterator as string)
    };
}