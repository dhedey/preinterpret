use preinterpret::*;

fn main() {
    stream! {
        [!set! #variable = Hello]
        [!set! #variable += World [!set! #variable += !]]
    }
}