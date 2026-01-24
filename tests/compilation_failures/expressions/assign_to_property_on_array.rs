use preinterpret::*;

fn main() {
    run!{
        let arr = [1, 2, 3];
        arr.xyz = "hello";
    };
}
