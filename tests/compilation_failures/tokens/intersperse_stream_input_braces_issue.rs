use preinterpret::*;

fn main() {
    preinterpret! {
        [!intersperse! {
            items: { 1 2 3 },
            separator: []
        }]
    }
}