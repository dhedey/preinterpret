use preinterpret::*;

macro_rules! assert_is_100 {
    ($input:literal) => {preinterpret!{
        [!if! ($input != 100) {
            [!error! [!string! "Expected 100, got " $input] [$input]]
        }]
    }};
}

fn main() {
    assert_is_100!(5);
}