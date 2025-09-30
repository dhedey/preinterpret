use preinterpret::*;

fn main() {
    let _ = stream! {
        #(partial_sum = [!stream! + 2]; 5 #..partial_sum)
    };
}
