use preinterpret::*;

fn main() {
    let _ = stream!{
        #(
            1 + 2 + 3 + 4;
            "This gets returned"
        )
    };
}