use preinterpret::*;

fn main() {
    let _ = stream!{
        #(let [_, _, _] = [1, 2])
    };
}