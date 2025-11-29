use preinterpret::stream;

stream! {
    #{
        let bytes = %[32];
        let postfix = %[Hello World #bytes];
        let my_raw_var = %raw[Test no #str [!ident! replacement]];
    }
    struct MyStruct;
    type #(%[X "Boo" Hello 1 #postfix].to_ident()) = MyStruct;
    const NUM: u32 = #(%[1337u #bytes].to_literal());
    const STRING: &str = #(my_raw_var.to_string());
    const SNAKE_CASE: &str = #(%[MyVar].to_string().to_lower_snake_case());
}

#[test]
fn complex_example_evaluates_correctly() {
    let _x: XBooHello1HelloWorld32 = MyStruct;
    assert_eq!(NUM, 1337u32);
    assert_eq!(STRING, "Testno#str[!ident!replacement]");
    assert_eq!(SNAKE_CASE, "my_var");
}
