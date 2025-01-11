use preinterpret::*;

macro_rules! assert_is_100 {
    ($input:literal) => {preinterpret!{
        [!if! ($input != 100) {
            [!error! [!string! "Expected 100, got " $input] [$input]]
        }]
    }};
}

fn main() {
    // In rust analyzer, the span is in the correct location
    // But in rustc it's not... I'm not really sure why.
    assert_is_100!(5);
}