use preinterpret::*;

fn main() {
    run!{
        None.configure_preinterpret(%{ iteration_limit: 5 });
        loop {}
    };
}