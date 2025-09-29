use preinterpret::*;

fn main() {
    let _ = stream! {
        #( 5 + [!set! #x = 2] 2)
    };
}
