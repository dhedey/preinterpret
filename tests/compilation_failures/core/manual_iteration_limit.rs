use preinterpret::*;

fn main() {
    run!{
        preinterpret::set_iteration_limit(4);
        loop {}
    };
}