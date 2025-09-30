use preinterpret::*;

macro_rules! assert_literals_eq_no_spans {
    ($input1:literal and $input2:literal) => {stream!{
        [!if! ($input1 != $input2) {
            [!error! %{
                message: [!string! "Expected " $input1 " to equal " $input2],
            }]
        }]
    }};
}

fn main() {
    assert_literals_eq_no_spans!(102 and 64);
}