#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

preinterpret! {
    [!set! #bytes = 32]
    [!set! #postfix = Hello World #bytes]
    [!set! #some_symbols = and some symbols such as [!raw! #] and #123]
    [!set! #MyRawVar = [!raw! Test no #str [!ident! replacement]]]
    [!ignore! non - sensical !code :D - ignored (!)]
    struct MyStruct;
    type [!ident! X "Boo" [!string! Hello 1] #postfix] = MyStruct;
    const NUM: u32 = [!literal! 1337u #bytes];
    const STRING: &str = [!string! #MyRawVar];
    const SNAKE_CASE: &str = [!snake! MyVar];
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
    assert_eq!(STRING, "Testno#str[!ident!replacement]");
    assert_eq!(SNAKE_CASE, "my_var");
}
