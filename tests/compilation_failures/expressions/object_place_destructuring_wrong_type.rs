use preinterpret::*;

fn main() {
    let _ = stream!{
        #(
            let x = 0;
            %{ x } = [x];
            x
        )
    };
}