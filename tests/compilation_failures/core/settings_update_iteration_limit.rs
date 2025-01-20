use preinterpret::*;

fn main() {
    preinterpret!{
        [!settings! { iteration_limit: 5 }]
        [!range! 0..100]
    };
}