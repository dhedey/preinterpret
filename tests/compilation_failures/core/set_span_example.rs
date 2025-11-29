use preinterpret::*;

// Taken from core.rs
macro_rules! capitalize_variants {
    ($enum_name:ident, [$($variants:ident),*]) => {run!{
        let enum_name = %raw[$enum_name];
        let variants = [];
        for variant in [$(%raw[$variants]),*] {
            let capitalized = variant.to_string().capitalize().to_ident();
            capitalized.set_span(variant);
            variants.push(capitalized);
        }

        %[
            enum #enum_name {
                #(variants.intersperse(%[,]))
            }
        ]
    }};
}

capitalize_variants!(MyEnum, [helloWorld, Test, HelloWorld]);

fn main() {}
