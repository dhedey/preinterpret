use preinterpret::*;

macro_rules! assert_literals_eq {
    ($input1:literal and $input2:literal) => {preinterpret!{
        [!if! ($input1 != $input2) {
            [!error! {
                message: [!string! "Expected " $input1 " to equal " $input2],
                spans: [$input1, $input2],
            }]
        }]
    }};
}

fn main() {
    assert_literals_eq!(102 and 64);
}