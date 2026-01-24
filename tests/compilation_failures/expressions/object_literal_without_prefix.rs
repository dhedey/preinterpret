use preinterpret::*;

fn main() {
    run! {
        [1, 2].intersperse("", { final_separator: "" })
    }
}