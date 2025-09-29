use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        #(-(-127i8 - 1))
    };
}