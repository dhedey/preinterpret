use preinterpret::*;

fn main() {
    let _ = preinterpret! {
        #(partial_sum = [!stream! + 2]; 5 #..partial_sum)
    };
}
