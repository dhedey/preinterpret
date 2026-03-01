use preinterpret::*;

fn main() {
    run!{
        let f = |g| g(g);
        f(f);
    }
}