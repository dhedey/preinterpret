use preinterpret::*;

fn main() {
    let _ = stream!{
        // zip is a method on objects, so using it as an identifier key should error
        #(%{ zip: 5 })
    };
}
