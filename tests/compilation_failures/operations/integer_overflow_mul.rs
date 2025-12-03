use preinterpret::*;

fn main() {
    let _ = stream!{
        #(200u8 * 2u8)
    };
}
