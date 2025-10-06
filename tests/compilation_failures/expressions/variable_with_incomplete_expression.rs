use preinterpret::*;

fn main() {
    let _ = stream!{
        #(x = %[+ 1]; 1 x)
    };
}