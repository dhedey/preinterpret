use preinterpret::*;

fn main() {
    preinterpret! {
        [!set! #variable = Hello]
        [!extend! #variable += World [!extend! #variable += !]]
    }
}