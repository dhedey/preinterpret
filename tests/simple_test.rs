use preinterpret::preinterpret;

preinterpret! {
    [!set! #bytes = 32]
    [!set! #postfix = Hello World #bytes]
    [!set! #MyRawVar = [!raw! Test no #str [!ident! replacement]]]
    struct MyStruct;
    type [!ident! X "Boo" [!string! Hello 1] #postfix] = MyStruct;
    const MY_NUM: u32 = [!literal! 1337u #bytes];
    const MY_STRING: &'static str = [!string! #MyRawVar];
}

#[test]
fn complex_example_evaluates_correctly() {
    let _x: XBooHello1HelloWorld32 = MyStruct;
    assert_eq!(MY_NUM, 1337u32);
    assert_eq!(MY_STRING, "Testno#str[!ident!replacement]");
}
