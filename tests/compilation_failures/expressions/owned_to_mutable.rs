use preinterpret::*;

fn main() {
    let _ = stream!{
        #(
            let a = "a";
            a.swap("b");
            a
        )
    };
}