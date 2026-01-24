use preinterpret::*;

fn main() {
    run!{
        let obj = %{};
        obj["false_key"] += 1;
        let _ = obj;
    };
}
