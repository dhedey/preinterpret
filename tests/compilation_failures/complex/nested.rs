use preinterpret::*;

fn main() {
    stream!([!if! true {
        [!if! true {
            [!if! true {
                [!error! %{
                    // Missing message
                }]
            }]
        }]
    }]);
} 