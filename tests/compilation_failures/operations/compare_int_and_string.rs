use preinterpret::*;

fn main() {
    let _ = stream!{
        #(5 == "five")
    };
}
