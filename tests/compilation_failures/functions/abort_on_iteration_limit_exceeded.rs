use preinterpret::*;

fn main() {
    run!{
        preinterpret::set_iteration_limit(14);
        for i in 0..15 {}
    }
}