use preinterpret::*;

fn main() {
    let _ = stream!{
        #(-(-127i8 - 1))
    };
}