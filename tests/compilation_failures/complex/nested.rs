use preinterpret::*;

fn main() {
    preinterpret!([!if! true {
        [!if! true {
            [!if! true {
                [!error! {
                    // Missing message
                }]
            }]
        }]
    }]);
} 