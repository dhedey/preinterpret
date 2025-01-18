use preinterpret::*;

fn main() {
    preinterpret! {
        [!intersperse! {
            items: [1 2],
            separator: [],
            add_trailing: {
                true false
            }
        }]
    }
}