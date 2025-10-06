use preinterpret::*;

macro_rules! assert_is_100 {
    ($input:literal) => {stream!{
        [!if! ($input != 100) {
            #(%[$input].error(%["Expected 100, got " $input].to_string()))
        }]
    }};
}

fn main() {
    assert_is_100!(5);
}