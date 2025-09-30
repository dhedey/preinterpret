use preinterpret::*;

fn main() {
    stream!([!let! @(#..x = @IDENT) = Hello]);
}
