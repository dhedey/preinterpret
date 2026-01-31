use preinterpret::*;

fn main() {
    run!{
        preinterpret::set_recursion_limit(4);
        let factorial = |n, f| {
            if n == 1 {
                1
            } else {
                n * f(n - 1, f)
            }
        };
        %[_].assert_eq(factorial(5, factorial), 120);
    }
}