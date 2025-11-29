use preinterpret::*;

fn main() {
    run!({
        loop {
            continue 'outer;
        }
    });
}
