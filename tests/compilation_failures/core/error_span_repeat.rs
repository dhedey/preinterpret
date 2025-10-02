use preinterpret::*;

macro_rules! assert_input_length_of_3 {
    ($($input:literal)+) => {stream!{
        #(
            let input = %raw[$($input)+];
            let input_length = input.len();
        )
        [!if! input_length != 3 {
            #(input.error(%["Expected 3 inputs, got " #input_length].string()))
        }]
    }};
}

fn main() {
    assert_input_length_of_3!(42 101 666 1024);
}