use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        [!set! #value = 1]
        [!set! #indirect = [!raw! #value]]
        [!evaluate! { #indirect }]
    };
}