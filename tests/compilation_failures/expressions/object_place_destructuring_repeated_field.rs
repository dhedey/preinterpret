use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        #(
            let x = 0;
            { x, x } = { x: 1 };
            x
        )
    };
}