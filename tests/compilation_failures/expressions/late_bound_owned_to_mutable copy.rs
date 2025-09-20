use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        #(
            let a = "a";
            "b".swap(a);
            a
        )
    };
}