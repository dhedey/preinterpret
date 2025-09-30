use preinterpret::*;

fn main() {
    let _ = stream!{
        #(let { x } = 0;)
    };
}