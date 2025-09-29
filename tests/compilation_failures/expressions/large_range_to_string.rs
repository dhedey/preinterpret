use preinterpret::*;

fn main() {
    let _ = stream!{
        #((0..10000) as iterator as string)
    };
}