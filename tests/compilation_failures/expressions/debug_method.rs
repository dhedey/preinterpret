use preinterpret::*;

fn main() {
    let _ = run!{
        let x = [1, 2];
        x.debug()
    };
}