use preinterpret::*;

fn main() {
    run! {
        let my_array_push = |arr: &any, value| arr.push(value);
        let arr = [1, 2, 3];
        my_array_push(arr, 4);
    }
}