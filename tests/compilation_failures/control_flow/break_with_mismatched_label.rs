use preinterpret::*;

fn main() {
    run!({
        loop {
            break 'outer;
        }
    });
}
