use preinterpret::*;

fn main() {
    stream! {
        #(let x = %[1 2];)
        [!intersperse! %{
            items: #x,
            separator: []
        }]
    }
}