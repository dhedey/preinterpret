use preinterpret::*;

// Taken from core.rs
macro_rules! capitalize_variants {
    ($enum_name:ident, [$($variants:ident),*]) => {run!{
        let enum_name = %raw[$enum_name];
        let variant_code = %[];
        let _ = [!for! variant in [$(%raw[$variants]),*] {#(
            let uppercased = variant.to_string().capitalize().to_ident().with_span(variant);
            variant_code += %[#uppercased,];
        )}];

        %[
            enum #enum_name {
                #variant_code
            }
        ]
    }};
}

capitalize_variants!(MyEnum, [helloWorld, Test, HelloWorld]);

fn main() {}
