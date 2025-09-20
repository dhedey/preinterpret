use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        #(
            let a = "a";
            a.swap("b");
            a
        )
    };
}