use preinterpret::*;

fn main() {
    preinterpret!{
        [!set! #is_eager = false]
        let _ = [!evaluate! false && ([!set! #is_eager = true] true)];
        [!if! #is_eager {
            [!error! "The && expression is not evaluated lazily" []]
        }]
    }
}