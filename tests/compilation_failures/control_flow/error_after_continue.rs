use preinterpret::*;

fn main() {
    run!(
        let x = 0;
        let _ = [!while! true {
            #(x += 1)
            [!if! x == 3 {
                [!continue!]
            } !elif! x >= 3 {
                // This checks that the "continue" flag is consumed,
                // and future errors propagate correctly.
                #(%[_].error("And now we error"))
            }]
        }];
    );
}