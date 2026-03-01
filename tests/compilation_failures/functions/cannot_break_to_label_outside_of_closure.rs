use preinterpret::*;

fn main() {
    run! {
        'outer: loop {
            let f = || {
                break 'outer;
            };
            f();
            break 'outer;
        }
    }
}
