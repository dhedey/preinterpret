use preinterpret::*;

fn main() {
    stream!([!if! true {
        [!if! true {
            [!if! true {
                // Missing message
                #(%[].error())
            }]
        }]
    }]);
} 