use preinterpret::*;

fn main() {
    run!({
        'inner: loop {
            break 'outer;
        }
    });
}
