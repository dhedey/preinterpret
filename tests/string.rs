#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

#[test]
fn test_string() {
    assert_eq!(
        run!(%[my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?].to_string()),
        "my_MixedCaseSTRINGWhichis  #awesome  -whatdo youthink?"
    );
    assert_eq!(run!(%[UPPER].to_string()), "UPPER");
    assert_eq!(run!(%[lower].to_string()), "lower");
    assert_eq!(run!(%[lower_snake_case].to_string()), "lower_snake_case");
    assert_eq!(run!(%[UPPER_SNAKE_CASE].to_string()), "UPPER_SNAKE_CASE");
    assert_eq!(run!(%[lowerCamelCase].to_string()), "lowerCamelCase");
    assert_eq!(run!(%[UpperCamelCase].to_string()), "UpperCamelCase");
    assert_eq!(run!(%[Capitalized].to_string()), "Capitalized");
    assert_eq!(
        run!(%["THEY SAID: A quick brown fox jumps over the lazy dog."].to_string()),
        "THEY SAID: A quick brown fox jumps over the lazy dog."
    );
    assert_eq!(run!(%["hello_w🌎rld"].to_string()), "hello_w🌎rld");
    assert_eq!(run!(%["kebab-case"].to_string()), "kebab-case");
    assert_eq!(
        run!(%["~~h4xx0rZ <3 1337c0de"].to_string()),
        "~~h4xx0rZ <3 1337c0de"
    );
    assert_eq!(
        run!(%[PostgreSQLConnection].to_string()),
        "PostgreSQLConnection"
    );
    assert_eq!(
        run!(%[PostgreSqlConnection].to_string()),
        "PostgreSqlConnection"
    );
    assert_eq!(
        run!(%["U+000A LINE FEED (LF)"].to_string()),
        "U+000A LINE FEED (LF)"
    );
    assert_eq!(
        run!(%["\nThis\r\n is a\tmulti-line\nstring"].to_string()),
        "\nThis\r\n is a\tmulti-line\nstring"
    );
    assert_eq!(
        run!(%["  lots of _ space and  _whacky |c$ara_cte>>rs|"].to_string()),
        "  lots of _ space and  _whacky |c$ara_cte>>rs|"
    );
    assert_eq!(run!(%["über CöÖl"].to_string()), "über CöÖl");
    assert_eq!(run!(%["◌̈ubër Cöol"].to_string()), "◌̈ubër Cöol"); // The ë (and only the e) uses a post-fix combining character
    assert_eq!(run!(%["真是难以置信！"].to_string()), "真是难以置信！");
}

#[test]
fn test_upper() {
    assert_eq!(
        run!(%[my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?].to_string().to_uppercase()),
        "MY_MIXEDCASESTRINGWHICHIS  #AWESOME  -WHATDO YOUTHINK?"
    );
    assert_eq!(run!(%[UPPER].to_string().to_uppercase()), "UPPER");
    assert_eq!(run!(%[lower].to_string().to_uppercase()), "LOWER");
    assert_eq!(
        run!(%[lower_snake_case].to_string().to_uppercase()),
        "LOWER_SNAKE_CASE"
    );
    assert_eq!(
        run!(%[UPPER_SNAKE_CASE].to_string().to_uppercase()),
        "UPPER_SNAKE_CASE"
    );
    assert_eq!(
        run!(%[lowerCamelCase].to_string().to_uppercase()),
        "LOWERCAMELCASE"
    );
    assert_eq!(
        run!(%[UpperCamelCase].to_string().to_uppercase()),
        "UPPERCAMELCASE"
    );
    assert_eq!(
        run!(%[Capitalized].to_string().to_uppercase()),
        "CAPITALIZED"
    );
    assert_eq!(
        run!("THEY SAID: A quick brown fox jumps over the lazy dog.".to_uppercase()),
        "THEY SAID: A QUICK BROWN FOX JUMPS OVER THE LAZY DOG."
    );
    assert_eq!(run!("hello_w🌎rld".to_uppercase()), "HELLO_W🌎RLD");
    assert_eq!(run!("kebab-case".to_uppercase()), "KEBAB-CASE");
    assert_eq!(
        run!("~~h4xx0rZ <3 1337c0de".to_uppercase()),
        "~~H4XX0RZ <3 1337C0DE"
    );
    assert_eq!(
        run!("PostgreSQLConnection".to_uppercase()),
        "POSTGRESQLCONNECTION"
    );
    assert_eq!(
        run!("PostgreSqlConnection".to_uppercase()),
        "POSTGRESQLCONNECTION"
    );
    assert_eq!(
        run!("U+000A LINE FEED (LF)".to_uppercase()),
        "U+000A LINE FEED (LF)"
    );
    assert_eq!(
        run!("\nThis\r\n is a\tmulti-line\nstring".to_uppercase()),
        "\nTHIS\r\n IS A\tMULTI-LINE\nSTRING"
    );
    assert_eq!(
        run!("  lots of _ space and  _whacky |c$ara_cte>>rs|".to_uppercase()),
        "  LOTS OF _ SPACE AND  _WHACKY |C$ARA_CTE>>RS|"
    );
    assert_eq!(run!("über CöÖl".to_uppercase()), "ÜBER CÖÖL");
    assert_eq!(run!("◌̈ubër Cöol".to_uppercase()), "◌̈UBËR CÖOL"); // The ë (and only the e) uses a post-fix combining character
    assert_eq!(run!("真是难以置信！".to_uppercase()), "真是难以置信！");
}

#[test]
fn test_lower() {
    assert_eq!(
        run!(%[my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?].to_string().to_lowercase()),
        "my_mixedcasestringwhichis  #awesome  -whatdo youthink?"
    );
    assert_eq!(run!(%[UPPER].to_string().to_lowercase()), "upper");
    assert_eq!(run!(%[lower].to_string().to_lowercase()), "lower");
    assert_eq!(
        run!(%[lower_snake_case].to_string().to_lowercase()),
        "lower_snake_case"
    );
    assert_eq!(
        run!(%[UPPER_SNAKE_CASE].to_string().to_lowercase()),
        "upper_snake_case"
    );
    assert_eq!(
        run!(%[lowerCamelCase].to_string().to_lowercase()),
        "lowercamelcase"
    );
    assert_eq!(
        run!(%[UpperCamelCase].to_string().to_lowercase()),
        "uppercamelcase"
    );
    assert_eq!(
        run!(%[Capitalized].to_string().to_lowercase()),
        "capitalized"
    );
    assert_eq!(
        run!(%["THEY SAID: A quick brown fox jumps over the lazy dog."].to_string().to_lowercase()),
        "they said: a quick brown fox jumps over the lazy dog."
    );
    assert_eq!(
        run!(%["hello_w🌎rld"].to_string().to_lowercase()),
        "hello_w🌎rld"
    );
    assert_eq!(
        run!(%["kebab-case"].to_string().to_lowercase()),
        "kebab-case"
    );
    assert_eq!(
        run!(%["~~h4xx0rZ <3 1337c0de"].to_string().to_lowercase()),
        "~~h4xx0rz <3 1337c0de"
    );
    assert_eq!(
        run!(%[PostgreSQLConnection].to_string().to_lowercase()),
        "postgresqlconnection"
    );
    assert_eq!(
        run!(%[PostgreSqlConnection].to_string().to_lowercase()),
        "postgresqlconnection"
    );
    assert_eq!(
        run!(%["U+000A LINE FEED (LF)"].to_string().to_lowercase()),
        "u+000a line feed (lf)"
    );
    assert_eq!(
        run!(%["\nThis\r\n is a\tmulti-line\nstring"].to_string().to_lowercase()),
        "\nthis\r\n is a\tmulti-line\nstring"
    );
    assert_eq!(
        run!(%["  lots of _ space and  _whacky |c$ara_cte>>rs|"].to_string().to_lowercase()),
        "  lots of _ space and  _whacky |c$ara_cte>>rs|"
    );
    assert_eq!(run!(%["über CöÖl"].to_string().to_lowercase()), "über cööl");
    assert_eq!(
        run!(%["◌̈ubër Cööl"].to_string().to_lowercase()),
        "◌̈ubër cööl"
    ); // The ë (and only the e) uses a post-fix combining character
    assert_eq!(
        run!(%["真是难以置信！"].to_string().to_lowercase()),
        "真是难以置信！"
    );
}

#[test]
fn test_snake() {
    assert_eq!(
        run!(%[my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?].to_string().to_lower_snake_case()),
        "my_mixed_case_string_whichis_awesome_whatdo_youthink"
    );
    assert_eq!(run!(%[UPPER].to_string().to_lower_snake_case()), "upper");
    assert_eq!(run!(%[lower].to_string().to_lower_snake_case()), "lower");
    assert_eq!(
        run!(%[lower_snake_case].to_string().to_lower_snake_case()),
        "lower_snake_case"
    );
    assert_eq!(
        run!(%[UPPER_SNAKE_CASE].to_string().to_lower_snake_case()),
        "upper_snake_case"
    );
    assert_eq!(
        run!(%[lowerCamelCase].to_string().to_lower_snake_case()),
        "lower_camel_case"
    );
    assert_eq!(
        run!(%[UpperCamelCase].to_string().to_lower_snake_case()),
        "upper_camel_case"
    );
    assert_eq!(
        run!(%[Capitalized].to_string().to_lower_snake_case()),
        "capitalized"
    );
    assert_eq!(
        run!(%["THEY SAID: A quick brown fox jumps over the lazy dog."].to_string().to_lower_snake_case()),
        "they_said_a_quick_brown_fox_jumps_over_the_lazy_dog"
    );
    assert_eq!(
        run!(%["hello_w🌎rld"].to_string().to_lower_snake_case()),
        "hello_w_rld"
    );
    assert_eq!(
        run!(%["kebab-case"].to_string().to_lower_snake_case()),
        "kebab_case"
    );
    assert_eq!(
        run!(%["~~h4xx0rZ <3 1337c0de"].to_string().to_lower_snake_case()),
        "h4xx0r_z_3_1337c0de"
    );
    assert_eq!(
        run!(%[PostgreSQLConnection].to_string().to_lower_snake_case()),
        "postgre_sql_connection"
    );
    assert_eq!(
        run!(%[PostgreSqlConnection].to_string().to_lower_snake_case()),
        "postgre_sql_connection"
    );
    assert_eq!(
        run!(%["U+000A LINE FEED (LF)"].to_string().to_lower_snake_case()),
        "u_000a_line_feed_lf"
    );
    assert_eq!(
        run!(%["\nThis\r\n is a\tmulti-line\nstring"].to_string().to_lower_snake_case()),
        "this_is_a_multi_line_string"
    );
    assert_eq!(
        run!(%["  lots of _ space and  _whacky |c$ara_cte>>rs|"].to_string().to_lower_snake_case()),
        "lots_of_space_and_whacky_c_ara_cte_rs"
    );
    assert_eq!(
        run!(%["über CöÖl"].to_string().to_lower_snake_case()),
        "über_cö_öl"
    );
    assert_eq!(
        run!(%["◌̈ubër Cööl"].to_string().to_lower_snake_case()),
        "ube_r_cööl"
    ); // The ë (and only the e) uses a post-fix combining character
    assert_eq!(
        run!(%["真是难以置信！"].to_string().to_lower_snake_case()),
        "真是难以置信"
    );
}

#[test]
fn test_upper_snake() {
    assert_eq!(
        run!(%[my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?].to_string().to_upper_snake_case()),
        "MY_MIXED_CASE_STRING_WHICHIS_AWESOME_WHATDO_YOUTHINK"
    );
    assert_eq!(run!(%[UPPER].to_string().to_upper_snake_case()), "UPPER");
    assert_eq!(run!(%[lower].to_string().to_upper_snake_case()), "LOWER");
    assert_eq!(
        run!(%[lower_snake_case].to_string().to_upper_snake_case()),
        "LOWER_SNAKE_CASE"
    );
    assert_eq!(
        run!(%[UPPER_SNAKE_CASE].to_string().to_upper_snake_case()),
        "UPPER_SNAKE_CASE"
    );
    assert_eq!(
        run!(%[lowerCamelCase].to_string().to_upper_snake_case()),
        "LOWER_CAMEL_CASE"
    );
    assert_eq!(
        run!(%[UpperCamelCase].to_string().to_upper_snake_case()),
        "UPPER_CAMEL_CASE"
    );
    assert_eq!(
        run!(%[Capitalized].to_string().to_upper_snake_case()),
        "CAPITALIZED"
    );
    assert_eq!(
        run!(%["THEY SAID: A quick brown fox jumps over the lazy dog."].to_string().to_upper_snake_case()),
        "THEY_SAID_A_QUICK_BROWN_FOX_JUMPS_OVER_THE_LAZY_DOG"
    );
    assert_eq!(
        run!(%["hello_w🌎rld"].to_string().to_upper_snake_case()),
        "HELLO_W_RLD"
    );
    assert_eq!(
        run!(%["kebab-case"].to_string().to_upper_snake_case()),
        "KEBAB_CASE"
    );
    assert_eq!(
        run!(%["~~h4xx0rZ <3 1337c0de"].to_string().to_upper_snake_case()),
        "H4XX0R_Z_3_1337C0DE"
    );
    assert_eq!(
        run!(%[PostgreSQLConnection].to_string().to_upper_snake_case()),
        "POSTGRE_SQL_CONNECTION"
    );
    assert_eq!(
        run!(%[PostgreSqlConnection].to_string().to_upper_snake_case()),
        "POSTGRE_SQL_CONNECTION"
    );
    assert_eq!(
        run!(%["U+000A LINE FEED (LF)"].to_string().to_upper_snake_case()),
        "U_000A_LINE_FEED_LF"
    );
    assert_eq!(
        run!(%["\nThis\r\n is a\tmulti-line\nstring"].to_string().to_upper_snake_case()),
        "THIS_IS_A_MULTI_LINE_STRING"
    );
    assert_eq!(
        run!(%["  lots of _ space and  _whacky |c$ara_cte>>rs|"].to_string().to_upper_snake_case()),
        "LOTS_OF_SPACE_AND_WHACKY_C_ARA_CTE_RS"
    );
    assert_eq!(
        run!(%["über CöÖl"].to_string().to_upper_snake_case()),
        "ÜBER_CÖ_ÖL"
    );
    assert_eq!(
        run!(%["◌̈ubër Cöol"].to_string().to_upper_snake_case()),
        "UBE_R_CÖOL"
    ); // The ë (and only the e) uses a post-fix combining character
    assert_eq!(
        run!(%["真是难以置信！"].to_string().to_upper_snake_case()),
        "真是难以置信"
    );
}

#[test]
fn test_to_lower_kebab_case() {
    assert_eq!(
        run!(%[my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?].to_string().to_kebab_case()),
        "my-mixed-case-string-whichis-awesome-whatdo-youthink"
    );
    assert_eq!(run!(%[UPPER].to_string().to_kebab_case()), "upper");
    assert_eq!(run!(%[lower].to_string().to_kebab_case()), "lower");
    assert_eq!(
        run!(%[lower_snake_case].to_string().to_kebab_case()),
        "lower-snake-case"
    );
    assert_eq!(
        run!(%[UPPER_SNAKE_CASE].to_string().to_kebab_case()),
        "upper-snake-case"
    );
    assert_eq!(
        run!(%[lowerCamelCase].to_string().to_kebab_case()),
        "lower-camel-case"
    );
    assert_eq!(
        run!(%[UpperCamelCase].to_string().to_kebab_case()),
        "upper-camel-case"
    );
    assert_eq!(
        run!(%[Capitalized].to_string().to_kebab_case()),
        "capitalized"
    );
    assert_eq!(
        run!(%["THEY SAID: A quick brown fox jumps over the lazy dog."].to_string().to_kebab_case()),
        "they-said-a-quick-brown-fox-jumps-over-the-lazy-dog"
    );
    assert_eq!(
        run!(%["hello_w🌎rld"].to_string().to_kebab_case()),
        "hello-w-rld"
    );
    assert_eq!(
        run!(%["kebab-case"].to_string().to_kebab_case()),
        "kebab-case"
    );
    assert_eq!(
        run!(%["~~h4xx0rZ <3 1337c0de"].to_string().to_kebab_case()),
        "h4xx0r-z-3-1337c0de"
    );
    assert_eq!(
        run!(%[PostgreSQLConnection].to_string().to_kebab_case()),
        "postgre-sql-connection"
    );
    assert_eq!(
        run!(%[PostgreSqlConnection].to_string().to_kebab_case()),
        "postgre-sql-connection"
    );
    assert_eq!(
        run!(%["U+000A LINE FEED (LF)"].to_string().to_kebab_case()),
        "u-000a-line-feed-lf"
    );
    assert_eq!(
        run!(%["\nThis\r\n is a\tmulti-line\nstring"].to_string().to_kebab_case()),
        "this-is-a-multi-line-string"
    );
    assert_eq!(
        run!(%["  lots of _ space and  _whacky |c$ara_cte>>rs|"].to_string().to_kebab_case()),
        "lots-of-space-and-whacky-c-ara-cte-rs"
    );
    assert_eq!(
        run!(%["über CöÖl"].to_string().to_kebab_case()),
        "über-cö-öl"
    );
    assert_eq!(
        run!(%["◌̈ubër Cööl"].to_string().to_kebab_case()),
        "ube-r-cööl"
    ); // The ë (and only the e) uses a post-fix combining character
    assert_eq!(
        run!(%["真是难以置信！"].to_string().to_kebab_case()),
        "真是难以置信"
    );
}

#[test]
fn test_lower_camel() {
    assert_eq!(
        run!(%[my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?].to_string().to_lower_camel_case()),
        "myMixedCaseStringWhichisAwesomeWhatdoYouthink"
    );
    assert_eq!(run!(%[UPPER].to_string().to_lower_camel_case()), "upper");
    assert_eq!(run!(%[lower].to_string().to_lower_camel_case()), "lower");
    assert_eq!(
        run!(%[lower_snake_case].to_string().to_lower_camel_case()),
        "lowerSnakeCase"
    );
    assert_eq!(
        run!(%[UPPER_SNAKE_CASE].to_string().to_lower_camel_case()),
        "upperSnakeCase"
    );
    assert_eq!(
        run!(%[lowerCamelCase].to_string().to_lower_camel_case()),
        "lowerCamelCase"
    );
    assert_eq!(
        run!(%[UpperCamelCase].to_string().to_lower_camel_case()),
        "upperCamelCase"
    );
    assert_eq!(
        run!(%[Capitalized].to_string().to_lower_camel_case()),
        "capitalized"
    );
    assert_eq!(
        run!(%["THEY SAID: A quick brown fox jumps over the lazy dog."].to_string().to_lower_camel_case()),
        "theySaidAQuickBrownFoxJumpsOverTheLazyDog"
    );
    assert_eq!(
        run!(%["hello_w🌎rld"].to_string().to_lower_camel_case()),
        "helloWRld"
    );
    assert_eq!(
        run!(%["kebab-case"].to_string().to_lower_camel_case()),
        "kebabCase"
    );
    assert_eq!(
        run!(%["~~h4xx0rZ <3 1337c0de"].to_string().to_lower_camel_case()),
        "h4xx0rZ31337c0de"
    );
    assert_eq!(
        run!(%[PostgreSQLConnection].to_string().to_lower_camel_case()),
        "postgreSqlConnection"
    );
    assert_eq!(
        run!(%[PostgreSqlConnection].to_string().to_lower_camel_case()),
        "postgreSqlConnection"
    );
    assert_eq!(
        run!(%["U+000A LINE FEED (LF)"].to_string().to_lower_camel_case()),
        "u000aLineFeedLf"
    );
    assert_eq!(
        run!(%["\nThis\r\n is a\tmulti-line\nstring"].to_string().to_lower_camel_case()),
        "thisIsAMultiLineString"
    );
    assert_eq!(
        run!(%["  lots of _ space and  _whacky |c$ara_cte>>rs|"].to_string().to_lower_camel_case()),
        "lotsOfSpaceAndWhackyCAraCteRs"
    );
    assert_eq!(
        run!(%["über CöÖl"].to_string().to_lower_camel_case()),
        "überCöÖl"
    );
    assert_eq!(
        run!(%["◌̈ubër Cööl"].to_string().to_lower_camel_case()),
        "ubeRCööl"
    ); // The ë (and only the e) uses a post-fix combining character
    assert_eq!(
        run!(%["真是难以置信！"].to_string().to_lower_camel_case()),
        "真是难以置信"
    );
}

#[test]
fn test_upper_camel() {
    assert_eq!(
        run!(%[my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?].to_string().to_upper_camel_case()),
        "MyMixedCaseStringWhichisAwesomeWhatdoYouthink"
    );
    assert_eq!(run!(%[UPPER].to_string().to_upper_camel_case()), "Upper");
    assert_eq!(run!(%[lower].to_string().to_upper_camel_case()), "Lower");
    assert_eq!(
        run!(%[lower_snake_case].to_string().to_upper_camel_case()),
        "LowerSnakeCase"
    );
    assert_eq!(
        run!(%[UPPER_SNAKE_CASE].to_string().to_upper_camel_case()),
        "UpperSnakeCase"
    );
    assert_eq!(
        run!(%[lowerCamelCase].to_string().to_upper_camel_case()),
        "LowerCamelCase"
    );
    assert_eq!(
        run!(%[UpperCamelCase].to_string().to_upper_camel_case()),
        "UpperCamelCase"
    );
    assert_eq!(
        run!(%[Capitalized].to_string().to_upper_camel_case()),
        "Capitalized"
    );
    assert_eq!(
        run!(%["THEY SAID: A quick brown fox jumps over the lazy dog."].to_string().to_upper_camel_case()),
        "TheySaidAQuickBrownFoxJumpsOverTheLazyDog"
    );
    assert_eq!(
        run!(%["hello_w🌎rld"].to_string().to_upper_camel_case()),
        "HelloWRld"
    );
    assert_eq!(
        run!(%["kebab-case"].to_string().to_upper_camel_case()),
        "KebabCase"
    );
    assert_eq!(
        run!(%["~~h4xx0rZ <3 1337c0de"].to_string().to_upper_camel_case()),
        "H4xx0rZ31337c0de"
    );
    assert_eq!(
        run!(%[PostgreSQLConnection].to_string().to_upper_camel_case()),
        "PostgreSqlConnection"
    );
    assert_eq!(
        run!(%[PostgreSqlConnection].to_string().to_upper_camel_case()),
        "PostgreSqlConnection"
    );
    assert_eq!(
        run!(%["U+000A LINE FEED (LF)"].to_string().to_upper_camel_case()),
        "U000aLineFeedLf"
    );
    assert_eq!(
        run!(%["\nThis\r\n is a\tmulti-line\nstring"].to_string().to_upper_camel_case()),
        "ThisIsAMultiLineString"
    );
    assert_eq!(
        run!(%["  lots of _ space and  _whacky |c$ara_cte>>rs|"].to_string().to_upper_camel_case()),
        "LotsOfSpaceAndWhackyCAraCteRs"
    );
    assert_eq!(
        run!(%["über CöÖl"].to_string().to_upper_camel_case()),
        "ÜberCöÖl"
    );
    assert_eq!(
        run!(%["◌̈ubër Cööl"].to_string().to_upper_camel_case()),
        "UbeRCööl"
    ); // The ë (and only the e) uses a post-fix combining character
    assert_eq!(
        run!(%["真是难以置信！"].to_string().to_upper_camel_case()),
        "真是难以置信"
    );
}

#[test]
fn test_capitalize() {
    assert_eq!(
        run!(%[my_ MixedCase STRING Which is "  #awesome  " - what do you think?].to_string().capitalize()),
        "My_MixedCaseSTRINGWhichis  #awesome  -whatdoyouthink?"
    );
    assert_eq!(run!(%[UPPER].to_string().capitalize()), "UPPER");
    assert_eq!(run!(%[lower].to_string().capitalize()), "Lower");
    assert_eq!(
        run!(%[lower_snake_case].to_string().capitalize()),
        "Lower_snake_case"
    );
    assert_eq!(
        run!(%[UPPER_SNAKE_CASE].to_string().capitalize()),
        "UPPER_SNAKE_CASE"
    );
    assert_eq!(
        run!(%[lowerCamelCase].to_string().capitalize()),
        "LowerCamelCase"
    );
    assert_eq!(
        run!(%[UpperCamelCase].to_string().capitalize()),
        "UpperCamelCase"
    );
    assert_eq!(run!(%[Capitalized].to_string().capitalize()), "Capitalized");
    assert_eq!(
        run!(%["THEY SAID: A quick brown fox jumps over the lazy dog."].to_string().capitalize()),
        "THEY SAID: A quick brown fox jumps over the lazy dog."
    );
    assert_eq!(
        run!(%["hello_w🌎rld"].to_string().capitalize()),
        "Hello_w🌎rld"
    );
    assert_eq!(run!(%["kebab-case"].to_string().capitalize()), "Kebab-case");
    assert_eq!(
        run!(%["~~h4xx0rZ <3 1337c0de"].to_string().capitalize()),
        "~~H4xx0rZ <3 1337c0de"
    );
    assert_eq!(
        run!(%[PostgreSQLConnection].to_string().capitalize()),
        "PostgreSQLConnection"
    );
    assert_eq!(
        run!(%[PostgreSqlConnection].to_string().capitalize()),
        "PostgreSqlConnection"
    );
    assert_eq!(
        run!(%["U+000A LINE FEED (LF)"].to_string().capitalize()),
        "U+000A LINE FEED (LF)"
    );
    assert_eq!(
        run!(%["\nThis\r\n is a\tmulti-line\nstring"].to_string().capitalize()),
        "\nThis\r\n is a\tmulti-line\nstring"
    );
    assert_eq!(
        run!(%["  lots of _ space and  _whacky |c$ara_cte>>rs|"].to_string().capitalize()),
        "  Lots of _ space and  _whacky |c$ara_cte>>rs|"
    );
    assert_eq!(run!(%["über CöÖl"].to_string().capitalize()), "Über CöÖl");
    assert_eq!(run!(%["◌̈ubër Cööl"].to_string().capitalize()), "◌̈Ubër Cööl"); // The ë (and only the e) uses a post-fix combining character
    assert_eq!(
        run!(%["真是难以置信！"].to_string().capitalize()),
        "真是难以置信！"
    );
}

#[test]
fn test_decapitalize() {
    assert_eq!(
        run!(%[my_ MixedCase STRING Which is "  #awesome  " - what do you think?].to_string().decapitalize()),
        "my_MixedCaseSTRINGWhichis  #awesome  -whatdoyouthink?"
    );
    assert_eq!(run!(%[UPPER].to_string().decapitalize()), "uPPER");
    assert_eq!(run!(%[lower].to_string().decapitalize()), "lower");
    assert_eq!(
        run!(%[lower_snake_case].to_string().decapitalize()),
        "lower_snake_case"
    );
    assert_eq!(
        run!(%[UPPER_SNAKE_CASE].to_string().decapitalize()),
        "uPPER_SNAKE_CASE"
    );
    assert_eq!(
        run!(%[lowerCamelCase].to_string().decapitalize()),
        "lowerCamelCase"
    );
    assert_eq!(
        run!(%[UpperCamelCase].to_string().decapitalize()),
        "upperCamelCase"
    );
    assert_eq!(
        run!(%[Capitalized].to_string().decapitalize()),
        "capitalized"
    );
    assert_eq!(
        run!(%["THEY SAID: A quick brown fox jumps over the lazy dog."].to_string().decapitalize()),
        "tHEY SAID: A quick brown fox jumps over the lazy dog."
    );
    assert_eq!(
        run!(%["hello_w🌎rld"].to_string().decapitalize()),
        "hello_w🌎rld"
    );
    assert_eq!(
        run!(%["kebab-case"].to_string().decapitalize()),
        "kebab-case"
    );
    assert_eq!(
        run!(%["~~h4xx0rZ <3 1337c0de"].to_string().decapitalize()),
        "~~h4xx0rZ <3 1337c0de"
    );
    assert_eq!(
        run!(%[PostgreSQLConnection].to_string().decapitalize()),
        "postgreSQLConnection"
    );
    assert_eq!(
        run!(%[PostgreSqlConnection].to_string().decapitalize()),
        "postgreSqlConnection"
    );
    assert_eq!(
        run!(%["U+000A LINE FEED (LF)"].to_string().decapitalize()),
        "u+000A LINE FEED (LF)"
    );
    assert_eq!(
        run!(%["\nThis\r\n is a\tmulti-line\nstring"].to_string().decapitalize()),
        "\nthis\r\n is a\tmulti-line\nstring"
    );
    assert_eq!(
        run!(%["  lots of _ space and  _whacky |c$ara_cte>>rs|"].to_string().decapitalize()),
        "  lots of _ space and  _whacky |c$ara_cte>>rs|"
    );
    assert_eq!(run!(%["über CöÖl"].to_string().decapitalize()), "über CöÖl");
    assert_eq!(
        run!(%["◌̈ubër Cööl"].to_string().decapitalize()),
        "◌̈ubër Cööl"
    ); // The ë (and only the e) uses a post-fix combining character
    assert_eq!(
        run!(%["真是难以置信！"].to_string().decapitalize()),
        "真是难以置信！"
    );
}

#[test]
fn test_title() {
    assert_eq!(
        run!(%[my_ MixedCase STRING Which is "  #awesome  " - what do you think?].to_string().to_title_case()),
        "My Mixed Case String Whichis Awesome Whatdoyouthink"
    );
    assert_eq!(run!(%[UPPER].to_string().to_title_case()), "Upper");
    assert_eq!(run!(%[lower].to_string().to_title_case()), "Lower");
    assert_eq!(
        run!(%[lower_snake_case].to_string().to_title_case()),
        "Lower Snake Case"
    );
    assert_eq!(
        run!(%[UPPER_SNAKE_CASE].to_string().to_title_case()),
        "Upper Snake Case"
    );
    assert_eq!(
        run!(%[lowerCamelCase].to_string().to_title_case()),
        "Lower Camel Case"
    );
    assert_eq!(
        run!(%[UpperCamelCase].to_string().to_title_case()),
        "Upper Camel Case"
    );
    assert_eq!(
        run!(%[Capitalized].to_string().to_title_case()),
        "Capitalized"
    );
    assert_eq!(
        run!(%["THEY SAID: A quick brown fox jumps over the lazy dog."].to_string().to_title_case()),
        "They Said A Quick Brown Fox Jumps Over The Lazy Dog"
    );
    assert_eq!(
        run!(%["hello_w🌎rld"].to_string().to_title_case()),
        "Hello W Rld"
    );
    assert_eq!(
        run!(%["kebab-case"].to_string().to_title_case()),
        "Kebab Case"
    );
    assert_eq!(
        run!(%["~~h4xx0rZ <3 1337c0de"].to_string().to_title_case()),
        "H4xx0r Z 3 1337c0de"
    );
    assert_eq!(
        run!(%[PostgreSQLConnection].to_string().to_title_case()),
        "Postgre Sql Connection"
    );
    assert_eq!(
        run!(%[PostgreSqlConnection].to_string().to_title_case()),
        "Postgre Sql Connection"
    );
    assert_eq!(
        run!(%["U+000A LINE FEED (LF)"].to_string().to_title_case()),
        "U 000a Line Feed Lf"
    );
    assert_eq!(
        run!(%["\nThis\r\n is a\tmulti-line\nstring"].to_string().to_title_case()),
        "This Is A Multi Line String"
    );
    assert_eq!(
        run!(%["  lots of _ space and  _whacky |c$ara_cte>>rs|"].to_string().to_title_case()),
        "Lots Of Space And Whacky C Ara Cte Rs"
    );
    assert_eq!(
        run!(%["über CöÖl"].to_string().to_title_case()),
        "Über Cö Öl"
    );
    assert_eq!(
        run!(%["◌̈ubër Cööl"].to_string().to_title_case()),
        "Ube R Cööl"
    ); // The ë (and only the e) uses a post-fix combining character
    assert_eq!(
        run!(%["真是难以置信！"].to_string().to_title_case()),
        "真是难以置信"
    );
}

#[test]
fn test_insert_spaces() {
    assert_eq!(
        run!(%[my_ MixedCase STRING Which is "  #awesome  " - what do you think?].to_string().insert_spaces()),
        "my Mixed Case STRING Whichis awesome whatdoyouthink"
    );
    assert_eq!(run!(%[UPPER].to_string().insert_spaces()), "UPPER");
    assert_eq!(run!(%[lower].to_string().insert_spaces()), "lower");
    assert_eq!(
        run!(%[lower_snake_case].to_string().insert_spaces()),
        "lower snake case"
    );
    assert_eq!(
        run!(%[UPPER_SNAKE_CASE].to_string().insert_spaces()),
        "UPPER SNAKE CASE"
    );
    assert_eq!(
        run!(%[lowerCamelCase].to_string().insert_spaces()),
        "lower Camel Case"
    );
    assert_eq!(
        run!(%[UpperCamelCase].to_string().insert_spaces()),
        "Upper Camel Case"
    );
    assert_eq!(
        run!(%[Capitalized].to_string().insert_spaces()),
        "Capitalized"
    );
    assert_eq!(
        run!(%["THEY SAID: A quick brown fox jumps over the lazy dog."].to_string().insert_spaces()),
        "THEY SAID A quick brown fox jumps over the lazy dog"
    );
    assert_eq!(
        run!(%["hello_w🌎rld"].to_string().insert_spaces()),
        "hello w rld"
    );
    assert_eq!(
        run!(%["kebab-case"].to_string().insert_spaces()),
        "kebab case"
    );
    assert_eq!(
        run!(%["~~h4xx0rZ <3 1337c0de"].to_string().insert_spaces()),
        "h4xx0r Z 3 1337c0de"
    );
    assert_eq!(
        run!(%[PostgreSQLConnection].to_string().insert_spaces()),
        "Postgre SQL Connection"
    );
    assert_eq!(
        run!(%[PostgreSqlConnection].to_string().insert_spaces()),
        "Postgre Sql Connection"
    );
    assert_eq!(
        run!(%["U+000A LINE FEED (LF)"].to_string().insert_spaces()),
        "U 000A LINE FEED LF"
    );
    assert_eq!(
        run!(%["\nThis\r\n is a\tmulti-line\nstring"].to_string().insert_spaces()),
        "This is a multi line string"
    );
    assert_eq!(
        run!(%["  lots of _ space and  _whacky |c$ara_cte>>rs|"].to_string().insert_spaces()),
        "lots of space and whacky c ara cte rs"
    );
    assert_eq!(
        run!(%["über CöÖl"].to_string().insert_spaces()),
        "über Cö Öl"
    );
    assert_eq!(
        run!(%["◌̈ubër Cööl"].to_string().insert_spaces()),
        "ube r Cööl"
    ); // The ë (and only the e) uses a post-fix combining character
    assert_eq!(
        run!(%["真是难以置信！"].to_string().insert_spaces()),
        "真是难以置信"
    );
}
