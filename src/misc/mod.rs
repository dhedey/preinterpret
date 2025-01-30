mod errors;
mod parse_traits;
mod string_conversion;

pub(crate) use errors::*;
pub(crate) use parse_traits::*;
pub(crate) use string_conversion::*;

#[allow(unused)]
pub(crate) fn print_if_slow<T>(
    inner: impl FnOnce() -> T,
    slow_threshold: std::time::Duration,
    print_message: impl FnOnce(&T, std::time::Duration) -> String,
) -> T {
    use std::time::*;
    let before = SystemTime::now();
    let output = inner();
    let after = SystemTime::now();

    let elapsed = after.duration_since(before).unwrap();
    if elapsed >= slow_threshold {
        println!("{}", print_message(&output, elapsed));
    }
    output
}
