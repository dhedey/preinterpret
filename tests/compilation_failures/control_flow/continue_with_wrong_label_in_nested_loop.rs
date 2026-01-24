use preinterpret::*;

fn main() {
    run!({
        'inner: loop {
            continue 'outer;
        }
    });
}
