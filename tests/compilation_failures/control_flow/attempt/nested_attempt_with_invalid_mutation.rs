use preinterpret::*;

fn main() {
    run!(
        let value = attempt {
            {
                let y = 1;
                attempt {
                    { y += 1; } => { y }
                }
            } => { y }
        };
    );
}