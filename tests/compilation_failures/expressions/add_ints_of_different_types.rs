use preinterpret::*;

fn main() {
    let _ = stream!{
        #(1u32 + 2u64)
    };
}