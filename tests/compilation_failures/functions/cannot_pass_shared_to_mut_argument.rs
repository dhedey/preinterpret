use preinterpret::*;

fn main() {
    run! {
        let my_array_mut_fn = |arr: &mut any| { None };
        my_array_mut_fn([1, 2, 3].as_ref());
    }
}