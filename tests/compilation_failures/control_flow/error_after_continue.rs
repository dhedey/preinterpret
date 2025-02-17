use preinterpret::*;

fn main() {
    preinterpret!(
        #(let x = 0)
        [!while! true {
            #(x += 1)
            [!if! x == 3 {
                [!continue!]
            } !elif! x >= 3 {
                // This checks that the "continue" flag is consumed,
                // and future errors propagate correctly.
                [!error! {
                    message: "And now we error"
                }]
            }]
        }]
    );
}