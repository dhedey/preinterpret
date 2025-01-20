use preinterpret::*;

fn main() {
    preinterpret!(
        [!set! #x = 0]
        [!while! true {
            [!assign! #x += 1]
            [!if! #x == 3 {
                [!continue!]
            } !elif! #x >= 3 {
                // This checks that the "continue" flag is consumed,
                // and future errors propogate correctly.
                [!error! {
                    message: "And now we error"
                }]
            }]
        }]
    );
}