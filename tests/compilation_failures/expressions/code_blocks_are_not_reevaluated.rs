use preinterpret::*;

fn main() {
    let _ = stream!{
        #(
            let indirect = [!raw! [!error! "This was a re-evaluation"]];
            // We don't get a re-evaluation. Instead, we get a parse error, because we end up
            // with let _ = [!error! "This was a re-evaluation"]; which is a parse error in
            // normal rust land.
            #(indirect)
        )
    };
}