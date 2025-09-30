use preinterpret::*;

fn main() {
    let _ = stream! {
        #(10 _ 10)
    };
}
