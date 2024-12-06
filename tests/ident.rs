use preinterpret::preinterpret;

macro_rules! my_assert_ident_eq {
    ($input:tt, $check:ident) => {{
        assert_eq!(
            {
                let preinterpret!($input) = 1;
                $check
            },
            1
        )
    }};
}

#[test]
#[allow(non_snake_case)]
fn test_ident() {
    my_assert_ident_eq!([!ident! a B C _D E], aBC_DE);
    my_assert_ident_eq!([!ident! a 12 "3"], a123);
    my_assert_ident_eq!([!ident! "MyString"], MyString);
    my_assert_ident_eq!([!ident! get_ [!snake! them]], get_them);
}

#[test]
#[allow(non_snake_case)]
fn test_ident_camel() {
    my_assert_ident_eq!([!ident_camel! a B C _D E], ABcDe);
    my_assert_ident_eq!([!ident_camel! a 12 "3"], A123);
    my_assert_ident_eq!([!ident_camel! "MyString"], MyString);
    my_assert_ident_eq!([!ident_camel! get_ [!snake! them]], GetThem);
    my_assert_ident_eq!([!ident_camel! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], MyMixedCaseStringWhichisAwesomeWhatdoYouthink);
}

#[test]
#[allow(non_snake_case)]
fn test_ident_snake() {
    my_assert_ident_eq!([!ident_snake! a B C _D E], a_bc_de);
    my_assert_ident_eq!([!ident_snake! a 12 "3"], a123);
    my_assert_ident_eq!([!ident_snake! "MyString"], my_string);
    my_assert_ident_eq!([!ident_snake! get_ [!snake! them]], get_them);
    my_assert_ident_eq!([!ident_snake! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], my_mixed_case_string_whichis_awesome_whatdo_youthink);
}

#[test]
#[allow(non_snake_case)]
fn test_ident_upper_snake() {
    my_assert_ident_eq!([!ident_upper_snake! a B C _D E], A_BC_DE);
    my_assert_ident_eq!([!ident_upper_snake! a 12 "3"], A123);
    my_assert_ident_eq!([!ident_upper_snake! "MyString"], MY_STRING);
    my_assert_ident_eq!([!ident_upper_snake! get_ [!snake! them]], GET_THEM);
    my_assert_ident_eq!([!ident_upper_snake! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], MY_MIXED_CASE_STRING_WHICHIS_AWESOME_WHATDO_YOUTHINK);
}
