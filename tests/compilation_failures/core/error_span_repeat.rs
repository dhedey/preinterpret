use preinterpret::*;

macro_rules! assert_input_length_of_3 {
    ($($input:literal)+) => {preinterpret!{
        [!set! #input_length = [!length! $($input)+]];
        [!if! (#input_length != 3) {
            [!error! {
                message: [!string! "Expected 3 inputs, got " #input_length],
                spans: [!stream! $($input)+],
            }]
        }]
    }};
}

fn main() {
    assert_input_length_of_3!(42 101 666 1024);
}