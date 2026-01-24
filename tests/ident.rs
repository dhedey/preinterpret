macro_rules! assert_ident {
    (($($input:tt)*), $check:ident) => {{
        assert_eq!(
            {
                let preinterpret::run!($($input)*) = 1;
                $check
            },
            1
        )
    }};
}

#[test]
#[allow(non_snake_case)]
fn test_ident() {
    assert_ident!((%ident[a B C _D E]), aBC_DE);
    assert_ident!((%[a B C _D E].to_ident()), aBC_DE);
    assert_ident!((%[a 12 "3"].to_ident()), a123);
    assert_ident!((%["MyString"].to_ident()), MyString);
    assert_ident!((%[get_ #(%[them].to_string().to_lower_snake_case())].to_ident()), get_them);
}

#[test]
#[allow(non_snake_case)]
fn test_ident_camel() {
    assert_ident!((%ident_camel[a B C _D E]), ABcDe);
    assert_ident!((%[a B C _D E].to_ident_camel()), ABcDe);
    assert_ident!((%[a 12 "3"].to_ident_camel()), A123);
    assert_ident!((%["MyString"].to_ident_camel()), MyString);
    assert_ident!((%[get_ them].to_ident_camel()), GetThem);
    assert_ident!((%[my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?].to_ident_camel()), MyMixedCaseStringWhichisAwesomeWhatdoYouthink);
}

#[test]
#[allow(non_snake_case)]
fn test_ident_snake() {
    assert_ident!((%ident_snake[a B C _D E]), a_bc_de);
    assert_ident!((%[a B C _D E].to_ident_snake()), a_bc_de);
    assert_ident!((%[a 12 "3"].to_ident_snake()), a123);
    assert_ident!((%["MyString"].to_ident_snake()), my_string);
    assert_ident!((%[get_ them].to_ident_snake()), get_them);
    assert_ident!((%[my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?].to_ident_snake()), my_mixed_case_string_whichis_awesome_whatdo_youthink);
}

#[test]
#[allow(non_snake_case)]
fn test_ident_upper_snake() {
    assert_ident!((%ident_upper_snake[a B C _D E]), A_BC_DE);
    assert_ident!((%[a B C _D E].to_ident_upper_snake()), A_BC_DE);
    assert_ident!((%[a 12 "3"].to_ident_upper_snake()), A123);
    assert_ident!((%["MyString"].to_ident_upper_snake()), MY_STRING);
    assert_ident!((%[get_ them].to_ident_upper_snake()), GET_THEM);
    assert_ident!((%[my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?].to_ident_upper_snake()), MY_MIXED_CASE_STRING_WHICHIS_AWESOME_WHATDO_YOUTHINK);
}
