use preinterpret::*;

fn main() {
    let _ = run! {
        let x = %{ a: 1 };
        // Technically x could be inactive here during the resolution
        // of its index, but for now, it's an error.
        x[{
            x.a += 1;
            "a"
        }] = 3;
    };
}