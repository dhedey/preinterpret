use preinterpret::*;

fn main() {
    let _ = stream!{
        #(%{ hello: "world", ["hello"]: "world_2" })
    };
}