use preinterpret::*;

fn main() {
    stream! {
        [!set! #x = 1 2]
        [!intersperse! %{
            items: #..x,
            separator: []
        }]
    }
}