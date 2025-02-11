use preinterpret::*;

fn main() {
    let _ = preinterpret! {
        #( 5 + [!set! #x = 2] 2)
    };
}
