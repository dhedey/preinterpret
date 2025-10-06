use preinterpret::*;

fn main() {
    stream!{
        [!settings! %{ iteration_limit: 5 }]
        [!loop! {}]
    };
}