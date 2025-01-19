use preinterpret::*;

fn main() {
    preinterpret!{
        [!set! #is_eager = false]
        let _ = [!evaluate! false && [!group! [!set! #is_eager = true] true]];
        [!if! #is_eager {
            [!error! { message: "The && expression is not evaluated lazily" }]
        }]
    }
}