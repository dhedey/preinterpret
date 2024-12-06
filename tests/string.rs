use preinterpret::preinterpret;

macro_rules! my_assert_eq {
    ($input:tt, $($output:tt)*) => {
        assert_eq!(preinterpret!($input), $($output)*);
    };
}

#[test]
fn test_string() {
    my_assert_eq!([!string! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "my_MixedCaseSTRINGWhichis  #awesome  -whatdo youthink?");
    my_assert_eq!([!string! UPPER], "UPPER");
    my_assert_eq!([!string! lower], "lower");
    my_assert_eq!([!string! lower_snake_case], "lower_snake_case");
    my_assert_eq!([!string! UPPER_SNAKE_CASE], "UPPER_SNAKE_CASE");
    my_assert_eq!([!string! lowerCamelCase], "lowerCamelCase");
    my_assert_eq!([!string! UpperCamelCase], "UpperCamelCase");
    my_assert_eq!([!string! Capitalized], "Capitalized");
    my_assert_eq!([!string! "THEY SAID: A quick brown fox jumps over the lazy dog."], "THEY SAID: A quick brown fox jumps over the lazy dog.");
    my_assert_eq!([!string! "hello_w🌎rld"], "hello_w🌎rld");
    my_assert_eq!([!string! "kebab-case"], "kebab-case");
    my_assert_eq!([!string! "~~h4xx0rZ <3 1337c0de"], "~~h4xx0rZ <3 1337c0de");
    my_assert_eq!([!string! PostgreSQLConnection], "PostgreSQLConnection");
    my_assert_eq!([!string! PostgreSqlConnection], "PostgreSqlConnection");
    my_assert_eq!([!string! "U+000A LINE FEED (LF)"], "U+000A LINE FEED (LF)");
    my_assert_eq!([!string! "\nThis\r\n is a\tmulti-line\nstring"], "\nThis\r\n is a\tmulti-line\nstring");
    my_assert_eq!([!string! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "  lots of _ space and  _whacky |c$ara_cte>>rs|");
    my_assert_eq!([!string! "über CöÖl"], "über CöÖl");
    my_assert_eq!([!string! "◌̈ubër Cöol"], "◌̈ubër Cöol"); // The ë (and only the e) uses a post-fix combining character
    my_assert_eq!([!string! "真是难以置信！"], "真是难以置信！");
}

#[test]
fn test_upper() {
    my_assert_eq!([!upper! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "MY_MIXEDCASESTRINGWHICHIS  #AWESOME  -WHATDO YOUTHINK?");
    my_assert_eq!([!upper! UPPER], "UPPER");
    my_assert_eq!([!upper! lower], "LOWER");
    my_assert_eq!([!upper! lower_snake_case], "LOWER_SNAKE_CASE");
    my_assert_eq!([!upper! UPPER_SNAKE_CASE], "UPPER_SNAKE_CASE");
    my_assert_eq!([!upper! lowerCamelCase], "LOWERCAMELCASE");
    my_assert_eq!([!upper! UpperCamelCase], "UPPERCAMELCASE");
    my_assert_eq!([!upper! Capitalized], "CAPITALIZED");
    my_assert_eq!([!upper! "THEY SAID: A quick brown fox jumps over the lazy dog."], "THEY SAID: A QUICK BROWN FOX JUMPS OVER THE LAZY DOG.");
    my_assert_eq!([!upper! "hello_w🌎rld"], "HELLO_W🌎RLD");
    my_assert_eq!([!upper! "kebab-case"], "KEBAB-CASE");
    my_assert_eq!([!upper! "~~h4xx0rZ <3 1337c0de"], "~~H4XX0RZ <3 1337C0DE");
    my_assert_eq!([!upper! PostgreSQLConnection], "POSTGRESQLCONNECTION");
    my_assert_eq!([!upper! PostgreSqlConnection], "POSTGRESQLCONNECTION");
    my_assert_eq!([!upper! "U+000A LINE FEED (LF)"], "U+000A LINE FEED (LF)");
    my_assert_eq!([!upper! "\nThis\r\n is a\tmulti-line\nstring"], "\nTHIS\r\n IS A\tMULTI-LINE\nSTRING");
    my_assert_eq!([!upper! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "  LOTS OF _ SPACE AND  _WHACKY |C$ARA_CTE>>RS|");
    my_assert_eq!([!upper! "über CöÖl"], "ÜBER CÖÖL");
    my_assert_eq!([!upper! "◌̈ubër Cöol"], "◌̈UBËR CÖOL"); // The ë (and only the e) uses a post-fix combining character
    my_assert_eq!([!upper! "真是难以置信！"], "真是难以置信！");
}

#[test]
fn test_lower() {
    my_assert_eq!([!lower! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "my_mixedcasestringwhichis  #awesome  -whatdo youthink?");
    my_assert_eq!([!lower! UPPER], "upper");
    my_assert_eq!([!lower! lower], "lower");
    my_assert_eq!([!lower! lower_snake_case], "lower_snake_case");
    my_assert_eq!([!lower! UPPER_SNAKE_CASE], "upper_snake_case");
    my_assert_eq!([!lower! lowerCamelCase], "lowercamelcase");
    my_assert_eq!([!lower! UpperCamelCase], "uppercamelcase");
    my_assert_eq!([!lower! Capitalized], "capitalized");
    my_assert_eq!([!lower! "THEY SAID: A quick brown fox jumps over the lazy dog."], "they said: a quick brown fox jumps over the lazy dog.");
    my_assert_eq!([!lower! "hello_w🌎rld"], "hello_w🌎rld");
    my_assert_eq!([!lower! "kebab-case"], "kebab-case");
    my_assert_eq!([!lower! "~~h4xx0rZ <3 1337c0de"], "~~h4xx0rz <3 1337c0de");
    my_assert_eq!([!lower! PostgreSQLConnection], "postgresqlconnection");
    my_assert_eq!([!lower! PostgreSqlConnection], "postgresqlconnection");
    my_assert_eq!([!lower! "U+000A LINE FEED (LF)"], "u+000a line feed (lf)");
    my_assert_eq!([!lower! "\nThis\r\n is a\tmulti-line\nstring"], "\nthis\r\n is a\tmulti-line\nstring");
    my_assert_eq!([!lower! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "  lots of _ space and  _whacky |c$ara_cte>>rs|");
    my_assert_eq!([!lower! "über CöÖl"], "über cööl");
    my_assert_eq!([!lower! "◌̈ubër Cööl"], "◌̈ubër cööl"); // The ë (and only the e) uses a post-fix combining character
    my_assert_eq!([!lower! "真是难以置信！"], "真是难以置信！");
}

#[test]
fn test_snake() {
    my_assert_eq!([!snake! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "my_mixed_case_string_whichis_awesome_whatdo_youthink");
    my_assert_eq!([!snake! UPPER], "upper");
    my_assert_eq!([!snake! lower], "lower");
    my_assert_eq!([!snake! lower_snake_case], "lower_snake_case");
    my_assert_eq!([!snake! UPPER_SNAKE_CASE], "upper_snake_case");
    my_assert_eq!([!snake! lowerCamelCase], "lower_camel_case");
    my_assert_eq!([!snake! UpperCamelCase], "upper_camel_case");
    my_assert_eq!([!snake! Capitalized], "capitalized");
    my_assert_eq!([!snake! "THEY SAID: A quick brown fox jumps over the lazy dog."], "they_said_a_quick_brown_fox_jumps_over_the_lazy_dog");
    my_assert_eq!([!snake! "hello_w🌎rld"], "hello_w_rld");
    my_assert_eq!([!snake! "kebab-case"], "kebab_case");
    my_assert_eq!([!snake! "~~h4xx0rZ <3 1337c0de"], "h4xx0r_z_3_1337c0de");
    my_assert_eq!([!snake! PostgreSQLConnection], "postgre_sql_connection");
    my_assert_eq!([!snake! PostgreSqlConnection], "postgre_sql_connection");
    my_assert_eq!([!snake! "U+000A LINE FEED (LF)"], "u_000a_line_feed_lf");
    my_assert_eq!([!snake! "\nThis\r\n is a\tmulti-line\nstring"], "this_is_a_multi_line_string");
    my_assert_eq!([!snake! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "lots_of_space_and_whacky_c_ara_cte_rs");
    my_assert_eq!([!snake! "über CöÖl"], "über_cö_öl");
    my_assert_eq!([!snake! "◌̈ubër Cööl"], "ube_r_cööl"); // The ë (and only the e) uses a post-fix combining character
    my_assert_eq!([!snake! "真是难以置信！"], "真是难以置信");
}

#[test]
fn test_upper_snake() {
    my_assert_eq!([!upper_snake! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "MY_MIXED_CASE_STRING_WHICHIS_AWESOME_WHATDO_YOUTHINK");
    my_assert_eq!([!upper_snake! UPPER], "UPPER");
    my_assert_eq!([!upper_snake! lower], "LOWER");
    my_assert_eq!([!upper_snake! lower_snake_case], "LOWER_SNAKE_CASE");
    my_assert_eq!([!upper_snake! UPPER_SNAKE_CASE], "UPPER_SNAKE_CASE");
    my_assert_eq!([!upper_snake! lowerCamelCase], "LOWER_CAMEL_CASE");
    my_assert_eq!([!upper_snake! UpperCamelCase], "UPPER_CAMEL_CASE");
    my_assert_eq!([!upper_snake! Capitalized], "CAPITALIZED");
    my_assert_eq!([!upper_snake! "THEY SAID: A quick brown fox jumps over the lazy dog."], "THEY_SAID_A_QUICK_BROWN_FOX_JUMPS_OVER_THE_LAZY_DOG");
    my_assert_eq!([!upper_snake! "hello_w🌎rld"], "HELLO_W_RLD");
    my_assert_eq!([!upper_snake! "kebab-case"], "KEBAB_CASE");
    my_assert_eq!([!upper_snake! "~~h4xx0rZ <3 1337c0de"], "H4XX0R_Z_3_1337C0DE");
    my_assert_eq!([!upper_snake! PostgreSQLConnection], "POSTGRE_SQL_CONNECTION");
    my_assert_eq!([!upper_snake! PostgreSqlConnection], "POSTGRE_SQL_CONNECTION");
    my_assert_eq!([!upper_snake! "U+000A LINE FEED (LF)"], "U_000A_LINE_FEED_LF");
    my_assert_eq!([!upper_snake! "\nThis\r\n is a\tmulti-line\nstring"], "THIS_IS_A_MULTI_LINE_STRING");
    my_assert_eq!([!upper_snake! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "LOTS_OF_SPACE_AND_WHACKY_C_ARA_CTE_RS");
    my_assert_eq!([!upper_snake! "über CöÖl"], "ÜBER_CÖ_ÖL");
    my_assert_eq!([!upper_snake! "◌̈ubër Cöol"], "UBE_R_CÖOL"); // The ë (and only the e) uses a post-fix combining character
    my_assert_eq!([!upper_snake! "真是难以置信！"], "真是难以置信");
}

#[test]
fn test_to_lower_kebab_case() {
    my_assert_eq!([!kebab! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "my-mixed-case-string-whichis-awesome-whatdo-youthink");
    my_assert_eq!([!kebab! UPPER], "upper");
    my_assert_eq!([!kebab! lower], "lower");
    my_assert_eq!([!kebab! lower_snake_case], "lower-snake-case");
    my_assert_eq!([!kebab! UPPER_SNAKE_CASE], "upper-snake-case");
    my_assert_eq!([!kebab! lowerCamelCase], "lower-camel-case");
    my_assert_eq!([!kebab! UpperCamelCase], "upper-camel-case");
    my_assert_eq!([!kebab! Capitalized], "capitalized");
    my_assert_eq!([!kebab! "THEY SAID: A quick brown fox jumps over the lazy dog."], "they-said-a-quick-brown-fox-jumps-over-the-lazy-dog");
    my_assert_eq!([!kebab! "hello_w🌎rld"], "hello-w-rld");
    my_assert_eq!([!kebab! "kebab-case"], "kebab-case");
    my_assert_eq!([!kebab! "~~h4xx0rZ <3 1337c0de"], "h4xx0r-z-3-1337c0de");
    my_assert_eq!([!kebab! PostgreSQLConnection], "postgre-sql-connection");
    my_assert_eq!([!kebab! PostgreSqlConnection], "postgre-sql-connection");
    my_assert_eq!([!kebab! "U+000A LINE FEED (LF)"], "u-000a-line-feed-lf");
    my_assert_eq!([!kebab! "\nThis\r\n is a\tmulti-line\nstring"], "this-is-a-multi-line-string");
    my_assert_eq!([!kebab! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "lots-of-space-and-whacky-c-ara-cte-rs");
    my_assert_eq!([!kebab! "über CöÖl"], "über-cö-öl");
    my_assert_eq!([!kebab! "◌̈ubër Cööl"], "ube-r-cööl"); // The ë (and only the e) uses a post-fix combining character
    my_assert_eq!([!kebab! "真是难以置信！"], "真是难以置信");
}

#[test]
fn test_lower_camel() {
    my_assert_eq!([!lower_camel! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "myMixedCaseStringWhichisAwesomeWhatdoYouthink");
    my_assert_eq!([!lower_camel! UPPER], "upper");
    my_assert_eq!([!lower_camel! lower], "lower");
    my_assert_eq!([!lower_camel! lower_snake_case], "lowerSnakeCase");
    my_assert_eq!([!lower_camel! UPPER_SNAKE_CASE], "upperSnakeCase");
    my_assert_eq!([!lower_camel! lowerCamelCase], "lowerCamelCase");
    my_assert_eq!([!lower_camel! UpperCamelCase], "upperCamelCase");
    my_assert_eq!([!lower_camel! Capitalized], "capitalized");
    my_assert_eq!([!lower_camel! "THEY SAID: A quick brown fox jumps over the lazy dog."], "theySaidAQuickBrownFoxJumpsOverTheLazyDog");
    my_assert_eq!([!lower_camel! "hello_w🌎rld"], "helloWRld");
    my_assert_eq!([!lower_camel! "kebab-case"], "kebabCase");
    my_assert_eq!([!lower_camel! "~~h4xx0rZ <3 1337c0de"], "h4xx0rZ31337c0de");
    my_assert_eq!([!lower_camel! PostgreSQLConnection], "postgreSqlConnection");
    my_assert_eq!([!lower_camel! PostgreSqlConnection], "postgreSqlConnection");
    my_assert_eq!([!lower_camel! "U+000A LINE FEED (LF)"], "u000aLineFeedLf");
    my_assert_eq!([!lower_camel! "\nThis\r\n is a\tmulti-line\nstring"], "thisIsAMultiLineString");
    my_assert_eq!([!lower_camel! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "lotsOfSpaceAndWhackyCAraCteRs");
    my_assert_eq!([!lower_camel! "über CöÖl"], "überCöÖl");
    my_assert_eq!([!lower_camel! "◌̈ubër Cööl"], "ubeRCööl"); // The ë (and only the e) uses a post-fix combining character
    my_assert_eq!([!lower_camel! "真是难以置信！"], "真是难以置信");
}

#[test]
fn test_camel() {
    my_assert_eq!([!camel! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "MyMixedCaseStringWhichisAwesomeWhatdoYouthink");
    my_assert_eq!([!camel! UPPER], "Upper");
    my_assert_eq!([!camel! lower], "Lower");
    my_assert_eq!([!camel! lower_snake_case], "LowerSnakeCase");
    my_assert_eq!([!camel! UPPER_SNAKE_CASE], "UpperSnakeCase");
    my_assert_eq!([!camel! lowerCamelCase], "LowerCamelCase");
    my_assert_eq!([!camel! UpperCamelCase], "UpperCamelCase");
    my_assert_eq!([!camel! Capitalized], "Capitalized");
    my_assert_eq!([!camel! "THEY SAID: A quick brown fox jumps over the lazy dog."], "TheySaidAQuickBrownFoxJumpsOverTheLazyDog");
    my_assert_eq!([!camel! "hello_w🌎rld"], "HelloWRld");
    my_assert_eq!([!camel! "kebab-case"], "KebabCase");
    my_assert_eq!([!camel! "~~h4xx0rZ <3 1337c0de"], "H4xx0rZ31337c0de");
    my_assert_eq!([!camel! PostgreSQLConnection], "PostgreSqlConnection");
    my_assert_eq!([!camel! PostgreSqlConnection], "PostgreSqlConnection");
    my_assert_eq!([!camel! "U+000A LINE FEED (LF)"], "U000aLineFeedLf");
    my_assert_eq!([!camel! "\nThis\r\n is a\tmulti-line\nstring"], "ThisIsAMultiLineString");
    my_assert_eq!([!camel! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "LotsOfSpaceAndWhackyCAraCteRs");
    my_assert_eq!([!camel! "über CöÖl"], "ÜberCöÖl");
    my_assert_eq!([!camel! "◌̈ubër Cööl"], "UbeRCööl"); // The ë (and only the e) uses a post-fix combining character
    my_assert_eq!([!camel! "真是难以置信！"], "真是难以置信");
}

#[test]
fn test_capitalize() {
    my_assert_eq!([!capitalize! my_ MixedCase STRING Which is "  #awesome  " - what do you think?], "My_MixedCaseSTRINGWhichis  #awesome  -whatdoyouthink?");
    my_assert_eq!([!capitalize! UPPER], "UPPER");
    my_assert_eq!([!capitalize! lower], "Lower");
    my_assert_eq!([!capitalize! lower_snake_case], "Lower_snake_case");
    my_assert_eq!([!capitalize! UPPER_SNAKE_CASE], "UPPER_SNAKE_CASE");
    my_assert_eq!([!capitalize! lowerCamelCase], "LowerCamelCase");
    my_assert_eq!([!capitalize! UpperCamelCase], "UpperCamelCase");
    my_assert_eq!([!capitalize! Capitalized], "Capitalized");
    my_assert_eq!([!capitalize! "THEY SAID: A quick brown fox jumps over the lazy dog."], "THEY SAID: A quick brown fox jumps over the lazy dog.");
    my_assert_eq!([!capitalize! "hello_w🌎rld"], "Hello_w🌎rld");
    my_assert_eq!([!capitalize! "kebab-case"], "Kebab-case");
    my_assert_eq!([!capitalize! "~~h4xx0rZ <3 1337c0de"], "~~H4xx0rZ <3 1337c0de");
    my_assert_eq!([!capitalize! PostgreSQLConnection], "PostgreSQLConnection");
    my_assert_eq!([!capitalize! PostgreSqlConnection], "PostgreSqlConnection");
    my_assert_eq!([!capitalize! "U+000A LINE FEED (LF)"], "U+000A LINE FEED (LF)");
    my_assert_eq!([!capitalize! "\nThis\r\n is a\tmulti-line\nstring"], "\nThis\r\n is a\tmulti-line\nstring");
    my_assert_eq!([!capitalize! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "  Lots of _ space and  _whacky |c$ara_cte>>rs|");
    my_assert_eq!([!capitalize! "über CöÖl"], "Über CöÖl");
    my_assert_eq!([!capitalize! "◌̈ubër Cööl"], "◌̈Ubër Cööl"); // The ë (and only the e) uses a post-fix combining character
    my_assert_eq!([!capitalize! "真是难以置信！"], "真是难以置信！");
}

#[test]
fn test_decapitalize() {
    my_assert_eq!([!decapitalize! my_ MixedCase STRING Which is "  #awesome  " - what do you think?], "my_MixedCaseSTRINGWhichis  #awesome  -whatdoyouthink?");
    my_assert_eq!([!decapitalize! UPPER], "uPPER");
    my_assert_eq!([!decapitalize! lower], "lower");
    my_assert_eq!([!decapitalize! lower_snake_case], "lower_snake_case");
    my_assert_eq!([!decapitalize! UPPER_SNAKE_CASE], "uPPER_SNAKE_CASE");
    my_assert_eq!([!decapitalize! lowerCamelCase], "lowerCamelCase");
    my_assert_eq!([!decapitalize! UpperCamelCase], "upperCamelCase");
    my_assert_eq!([!decapitalize! Capitalized], "capitalized");
    my_assert_eq!([!decapitalize! "THEY SAID: A quick brown fox jumps over the lazy dog."], "tHEY SAID: A quick brown fox jumps over the lazy dog.");
    my_assert_eq!([!decapitalize! "hello_w🌎rld"], "hello_w🌎rld");
    my_assert_eq!([!decapitalize! "kebab-case"], "kebab-case");
    my_assert_eq!([!decapitalize! "~~h4xx0rZ <3 1337c0de"], "~~h4xx0rZ <3 1337c0de");
    my_assert_eq!([!decapitalize! PostgreSQLConnection], "postgreSQLConnection");
    my_assert_eq!([!decapitalize! PostgreSqlConnection], "postgreSqlConnection");
    my_assert_eq!([!decapitalize! "U+000A LINE FEED (LF)"], "u+000A LINE FEED (LF)");
    my_assert_eq!([!decapitalize! "\nThis\r\n is a\tmulti-line\nstring"], "\nthis\r\n is a\tmulti-line\nstring");
    my_assert_eq!([!decapitalize! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "  lots of _ space and  _whacky |c$ara_cte>>rs|");
    my_assert_eq!([!decapitalize! "über CöÖl"], "über CöÖl");
    my_assert_eq!([!decapitalize! "◌̈ubër Cööl"], "◌̈ubër Cööl"); // The ë (and only the e) uses a post-fix combining character
    my_assert_eq!([!decapitalize! "真是难以置信！"], "真是难以置信！");
}

#[test]
fn test_title() {
    my_assert_eq!([!title! my_ MixedCase STRING Which is "  #awesome  " - what do you think?], "My Mixed Case String Whichis Awesome Whatdoyouthink");
    my_assert_eq!([!title! UPPER], "Upper");
    my_assert_eq!([!title! lower], "Lower");
    my_assert_eq!([!title! lower_snake_case], "Lower Snake Case");
    my_assert_eq!([!title! UPPER_SNAKE_CASE], "Upper Snake Case");
    my_assert_eq!([!title! lowerCamelCase], "Lower Camel Case");
    my_assert_eq!([!title! UpperCamelCase], "Upper Camel Case");
    my_assert_eq!([!title! Capitalized], "Capitalized");
    my_assert_eq!([!title! "THEY SAID: A quick brown fox jumps over the lazy dog."], "They Said A Quick Brown Fox Jumps Over The Lazy Dog");
    my_assert_eq!([!title! "hello_w🌎rld"], "Hello W Rld");
    my_assert_eq!([!title! "kebab-case"], "Kebab Case");
    my_assert_eq!([!title! "~~h4xx0rZ <3 1337c0de"], "H4xx0r Z 3 1337c0de");
    my_assert_eq!([!title! PostgreSQLConnection], "Postgre Sql Connection");
    my_assert_eq!([!title! PostgreSqlConnection], "Postgre Sql Connection");
    my_assert_eq!([!title! "U+000A LINE FEED (LF)"], "U 000a Line Feed Lf");
    my_assert_eq!([!title! "\nThis\r\n is a\tmulti-line\nstring"], "This Is A Multi Line String");
    my_assert_eq!([!title! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "Lots Of Space And Whacky C Ara Cte Rs");
    my_assert_eq!([!title! "über CöÖl"], "Über Cö Öl");
    my_assert_eq!([!title! "◌̈ubër Cööl"], "Ube R Cööl"); // The ë (and only the e) uses a post-fix combining character
    my_assert_eq!([!title! "真是难以置信！"], "真是难以置信");
}

#[test]
fn test_insert_spaces() {
    my_assert_eq!([!insert_spaces! my_ MixedCase STRING Which is "  #awesome  " - what do you think?], "my Mixed Case STRING Whichis awesome whatdoyouthink");
    my_assert_eq!([!insert_spaces! UPPER], "UPPER");
    my_assert_eq!([!insert_spaces! lower], "lower");
    my_assert_eq!([!insert_spaces! lower_snake_case], "lower snake case");
    my_assert_eq!([!insert_spaces! UPPER_SNAKE_CASE], "UPPER SNAKE CASE");
    my_assert_eq!([!insert_spaces! lowerCamelCase], "lower Camel Case");
    my_assert_eq!([!insert_spaces! UpperCamelCase], "Upper Camel Case");
    my_assert_eq!([!insert_spaces! Capitalized], "Capitalized");
    my_assert_eq!([!insert_spaces! "THEY SAID: A quick brown fox jumps over the lazy dog."], "THEY SAID A quick brown fox jumps over the lazy dog");
    my_assert_eq!([!insert_spaces! "hello_w🌎rld"], "hello w rld");
    my_assert_eq!([!insert_spaces! "kebab-case"], "kebab case");
    my_assert_eq!([!insert_spaces! "~~h4xx0rZ <3 1337c0de"], "h4xx0r Z 3 1337c0de");
    my_assert_eq!([!insert_spaces! PostgreSQLConnection], "Postgre SQL Connection");
    my_assert_eq!([!insert_spaces! PostgreSqlConnection], "Postgre Sql Connection");
    my_assert_eq!([!insert_spaces! "U+000A LINE FEED (LF)"], "U 000A LINE FEED LF");
    my_assert_eq!([!insert_spaces! "\nThis\r\n is a\tmulti-line\nstring"], "This is a multi line string");
    my_assert_eq!([!insert_spaces! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "lots of space and whacky c ara cte rs");
    my_assert_eq!([!insert_spaces! "über CöÖl"], "über Cö Öl");
    my_assert_eq!([!insert_spaces! "◌̈ubër Cööl"], "ube r Cööl"); // The ë (and only the e) uses a post-fix combining character
    my_assert_eq!([!insert_spaces! "真是难以置信！"], "真是难以置信");
}
