use preinterpret::*;

fn main() {
    stream!([!let! @(#..x = @LITERAL) = "Hello"]);
}
