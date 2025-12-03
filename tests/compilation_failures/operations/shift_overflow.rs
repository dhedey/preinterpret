use preinterpret::*;

fn main() {
    let _ = stream!{
        #(1u8 << 8)
    };
}
