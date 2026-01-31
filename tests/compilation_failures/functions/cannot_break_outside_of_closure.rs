use preinterpret::*;

fn main() {
    run! {
        loop {
            let f = || {
                break;
            };
            f();
            break;
        }
    }
}
