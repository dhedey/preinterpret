use preinterpret::*;

fn main() {
    stream! {
        #(variable += %[2];)
    }
}