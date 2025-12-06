use preinterpret::*;

fn main() {
    let _ = run!{
        let a = "a";
        a.swap(a);
        a
    };
}