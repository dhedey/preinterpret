use preinterpret::*;

fn main() {
    let _ = stream!{
        #(_[0] = 10)
    };
}