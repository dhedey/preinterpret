use preinterpret::*;

fn main() {
    let _ = run!{
        let arr = [0, 1];
        arr[arr[1]] = 3;
    };
}