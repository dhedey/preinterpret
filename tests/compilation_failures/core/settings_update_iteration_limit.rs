use preinterpret::*;

fn main() {
    run!{
        [!settings! %{ iteration_limit: 5 }];
        loop {}
    };
}