use preinterpret::*;

fn main() {
    run! {
        let my_array_destructure = |arr: &mut any, value| {
            let [first, second] = arr;
            first + second
        };
        my_array_destructure([1, 2, 3].as_mut(), 4);
    }
}