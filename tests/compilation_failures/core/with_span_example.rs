use preinterpret::*;

// Taken from core.rs
macro_rules! capitalize_variants {
    ($enum_name:ident, [$($variants:ident),*]) => {run!{
        let enum_name = %raw[$enum_name];
        let variants = [];
        let _ = [!for! variant in [$(%raw[$variants]),*] {#({
            let capitalized = variant.to_string().capitalize().to_ident().with_span(variant);
            variants.push(capitalized.take_owned());
        })}];

        %[
            enum #enum_name {
                #(variants.take_owned().intersperse(%[,]))
            }
        ]
    }};
}

capitalize_variants!(MyEnum, [helloWorld, Test, HelloWorld]);

fn main() {}
