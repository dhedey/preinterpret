use preinterpret::*;

fn main() {
    let _ = stream!{
        #(x = [!stream! + 1]; 1 x)
    };
}