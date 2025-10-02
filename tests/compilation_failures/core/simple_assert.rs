use preinterpret::*;

macro_rules! assert_literals_eq {
    ($input1:literal, $input2:literal) => {run!{
        %[$input1 $input2].assert($input1 == $input2, %["Expected " $input1 " to equal " $input2].to_string());
    }};
}

fn main() {
    assert_literals_eq!(102, 64);
}