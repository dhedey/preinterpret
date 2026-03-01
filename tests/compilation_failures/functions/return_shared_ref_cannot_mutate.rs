use preinterpret::*;

fn main() {
    run! {
        let convert_to_ref = |x| x.as_ref();
        convert_to_ref("hello").as_mut()
    }
}
