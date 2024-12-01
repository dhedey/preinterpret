use preinterpret::preinterpret;

preinterpret! {
    [!set! #bytes = 32]
    [!set! #postfix = Hello World #bytes]
    [!set! #MyRawVar = [!raw! Test no #str [!ident! replacement]]]
    struct MyStruct;
    type [!ident! X "Boo" [!string! Hello 1] #postfix] = MyStruct;
    const NUM: u32 = [!literal! 1337u #bytes];
    const STRING: &str = [!string! #MyRawVar];
    const SNAKE_CASE: &str = [!snake_case! MyVar];
}

#[test]
fn complex_example_evaluates_correctly() {
    let _x: XBooHello1HelloWorld32 = MyStruct;
    assert_eq!(NUM, 1337u32);
    assert_eq!(STRING, "Testno#str[!ident!replacement]");
    assert_eq!(SNAKE_CASE, "my_var");
}
