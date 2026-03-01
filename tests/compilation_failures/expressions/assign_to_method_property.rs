use preinterpret::*;

fn main() {
    let _ = stream!{
        #({
            let obj = %{};
            // zip is a method on objects, so this should error
            obj.zip = 5;
        })
    };
}
