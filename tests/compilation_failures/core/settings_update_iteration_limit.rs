use preinterpret::*;

fn main() {
    preinterpret!{
        [!settings! { iteration_limit: 5 }]
        [!loop! {}]
    };
}