use preinterpret::*;

fn main() {
    let _ = stream!{
        #(
            let a = "a";
            "b".swap(a);
            a
        )
    };
}