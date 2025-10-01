use preinterpret::*;

fn main() {
    stream! {
        [!set! #variable = Hello]
        [!set! #variable += World #(variable = %[Hello2])]
    }
}