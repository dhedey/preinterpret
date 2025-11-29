use preinterpret::*;

fn main() {
    let _ = run!{
        let a = "a";
        "b".swap(a);
        a
    };
}