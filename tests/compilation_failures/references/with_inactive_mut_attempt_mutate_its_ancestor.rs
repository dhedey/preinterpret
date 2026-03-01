use preinterpret::*;

fn main() {
    let _ = run!{
        let my_arr = [[]];
        my_arr[0].push(my_arr.pop())
    };
}