use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        #(x = [!stream! + 1]; 1 x)
    };
}