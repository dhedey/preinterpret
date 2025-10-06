#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

preinterpret::run! {
    let bytes = 32;
    let postfix = %[Hello World #bytes];
    let some_symbols = %[and some symbols such as %raw[#] and #123];
    let MyRawVar = %raw[Test no #str $(%[replacement].to_ident())];
    let _ = %raw[non - sensical !code :D - ignored (!)];
    %[
        struct MyStruct;
        type #(%[X "Boo" #(%[Hello 1].to_string()) #postfix].to_ident()) = MyStruct;
        const NUM: u32 = #(%[1337u #bytes].to_literal());
        const STRING: &str = #(MyRawVar.to_string());
        const SNAKE_CASE: &str = #("MyVar".to_lower_snake_case());
    ]
}

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_complex_compilation_failures() {
    if !should_run_ui_tests() {
        // Some of the outputs are different on nightly, so don't test these
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/complex/*.rs");
}

#[test]
fn complex_example_evaluates_correctly() {
    let _x: XBooHello1HelloWorld32 = MyStruct;
    assert_eq!(NUM, 1337u32);
    assert_eq!(STRING, "Testno#str$(%[replacement].to_ident())");
    assert_eq!(SNAKE_CASE, "my_var");
}
