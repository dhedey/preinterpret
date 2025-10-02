use preinterpret::*;

macro_rules! assert_literals_eq {
    ($input1:literal and $input2:literal) => {stream!{
        [!if! ($input1 != $input2) {
            #(%[$input1 $input2].error(%["Expected " $input1 " to equal " $input2].to_string()))
        }]
    }};
}

fn main() {
    assert_literals_eq!(102 and 64);
}