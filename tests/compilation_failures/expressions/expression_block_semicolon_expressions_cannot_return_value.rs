use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        #(
            1 + 2 + 3 + 4;
            "This gets returned"
        )
    };
}