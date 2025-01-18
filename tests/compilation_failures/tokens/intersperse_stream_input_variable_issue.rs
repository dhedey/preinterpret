use preinterpret::*;

fn main() {
    preinterpret! {
        [!set! #x = 1 2]
        [!intersperse! {
            items: #..x,
            separator: []
        }]
    }
}