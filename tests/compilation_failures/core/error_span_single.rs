use preinterpret::*;

macro_rules! assert_is_100 {
    ($input:literal) => {preinterpret!{
        [!if! ($input != 100) {
            [!error! {
                message: [!string! "Expected 100, got " $input],
                spans: [!stream! $input],
            }]
        }]
    }};
}

fn main() {
    assert_is_100!(5);
}