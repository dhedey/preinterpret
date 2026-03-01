use preinterpret::*;

fn main() {
    let _ = run! {
        let x = %{ a: 1 };
        let f = |x_field: &any| {
            x.b = "new!!";
        };
        f(x.a);
    };
}