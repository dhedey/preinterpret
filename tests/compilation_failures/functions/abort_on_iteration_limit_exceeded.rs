use preinterpret::*;

fn main() {
    run!{
        let arr = [1, 2, 3];
        array::push_me(arr, 4, 5);
    }
}