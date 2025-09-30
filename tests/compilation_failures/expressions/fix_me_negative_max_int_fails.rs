use preinterpret::*;

fn main() {
    let _ = stream! {
        // This should not fail, and should be fixed.
        // This test just records the fact it doesn't work as a known issue.
        // A fix of this should remove this test and move it to a working test.
        #(-128i8)
        // This should also not fail according to the rules of rustc.
        #(-(--128i8))
    };
}