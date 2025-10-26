
use preinterpret::*;

fn main() {
    run! {
        attempt {
            { } if true => x
        }
    };
}
