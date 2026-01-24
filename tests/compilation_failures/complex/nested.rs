use preinterpret::*;

fn main() {
    run!(
        if true {
            if true {
                if true {
                    // Missing message
                    %[].error()
                }
            }
        }
    );
}