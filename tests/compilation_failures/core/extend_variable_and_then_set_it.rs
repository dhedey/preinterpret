use preinterpret::*;

fn main() {
    stream! {
        [!set! #variable = Hello]
        [!set! #variable += World #(variable = [!stream! Hello2])]
    }
}