#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

#[test]
fn test_string() {
    preinterpret_assert_eq!([!string! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "my_MixedCaseSTRINGWhichis  #awesome  -whatdo youthink?");
    preinterpret_assert_eq!([!string! UPPER], "UPPER");
    preinterpret_assert_eq!([!string! lower], "lower");
    preinterpret_assert_eq!([!string! lower_snake_case], "lower_snake_case");
    preinterpret_assert_eq!([!string! UPPER_SNAKE_CASE], "UPPER_SNAKE_CASE");
    preinterpret_assert_eq!([!string! lowerCamelCase], "lowerCamelCase");
    preinterpret_assert_eq!([!string! UpperCamelCase], "UpperCamelCase");
    preinterpret_assert_eq!([!string! Capitalized], "Capitalized");
    preinterpret_assert_eq!([!string! "THEY SAID: A quick brown fox jumps over the lazy dog."], "THEY SAID: A quick brown fox jumps over the lazy dog.");
    preinterpret_assert_eq!([!string! "hello_w🌎rld"], "hello_w🌎rld");
    preinterpret_assert_eq!([!string! "kebab-case"], "kebab-case");
    preinterpret_assert_eq!([!string! "~~h4xx0rZ <3 1337c0de"], "~~h4xx0rZ <3 1337c0de");
    preinterpret_assert_eq!([!string! PostgreSQLConnection], "PostgreSQLConnection");
    preinterpret_assert_eq!([!string! PostgreSqlConnection], "PostgreSqlConnection");
    preinterpret_assert_eq!([!string! "U+000A LINE FEED (LF)"], "U+000A LINE FEED (LF)");
    preinterpret_assert_eq!([!string! "\nThis\r\n is a\tmulti-line\nstring"], "\nThis\r\n is a\tmulti-line\nstring");
    preinterpret_assert_eq!([!string! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "  lots of _ space and  _whacky |c$ara_cte>>rs|");
    preinterpret_assert_eq!([!string! "über CöÖl"], "über CöÖl");
    preinterpret_assert_eq!([!string! "◌̈ubër Cöol"], "◌̈ubër Cöol"); // The ë (and only the e) uses a post-fix combining character
    preinterpret_assert_eq!([!string! "真是难以置信！"], "真是难以置信！");
}

#[test]
fn test_upper() {
    preinterpret_assert_eq!([!upper! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "MY_MIXEDCASESTRINGWHICHIS  #AWESOME  -WHATDO YOUTHINK?");
    preinterpret_assert_eq!([!upper! UPPER], "UPPER");
    preinterpret_assert_eq!([!upper! lower], "LOWER");
    preinterpret_assert_eq!([!upper! lower_snake_case], "LOWER_SNAKE_CASE");
    preinterpret_assert_eq!([!upper! UPPER_SNAKE_CASE], "UPPER_SNAKE_CASE");
    preinterpret_assert_eq!([!upper! lowerCamelCase], "LOWERCAMELCASE");
    preinterpret_assert_eq!([!upper! UpperCamelCase], "UPPERCAMELCASE");
    preinterpret_assert_eq!([!upper! Capitalized], "CAPITALIZED");
    preinterpret_assert_eq!([!upper! "THEY SAID: A quick brown fox jumps over the lazy dog."], "THEY SAID: A QUICK BROWN FOX JUMPS OVER THE LAZY DOG.");
    preinterpret_assert_eq!([!upper! "hello_w🌎rld"], "HELLO_W🌎RLD");
    preinterpret_assert_eq!([!upper! "kebab-case"], "KEBAB-CASE");
    preinterpret_assert_eq!([!upper! "~~h4xx0rZ <3 1337c0de"], "~~H4XX0RZ <3 1337C0DE");
    preinterpret_assert_eq!([!upper! PostgreSQLConnection], "POSTGRESQLCONNECTION");
    preinterpret_assert_eq!([!upper! PostgreSqlConnection], "POSTGRESQLCONNECTION");
    preinterpret_assert_eq!([!upper! "U+000A LINE FEED (LF)"], "U+000A LINE FEED (LF)");
    preinterpret_assert_eq!([!upper! "\nThis\r\n is a\tmulti-line\nstring"], "\nTHIS\r\n IS A\tMULTI-LINE\nSTRING");
    preinterpret_assert_eq!([!upper! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "  LOTS OF _ SPACE AND  _WHACKY |C$ARA_CTE>>RS|");
    preinterpret_assert_eq!([!upper! "über CöÖl"], "ÜBER CÖÖL");
    preinterpret_assert_eq!([!upper! "◌̈ubër Cöol"], "◌̈UBËR CÖOL"); // The ë (and only the e) uses a post-fix combining character
    preinterpret_assert_eq!([!upper! "真是难以置信！"], "真是难以置信！");
}

#[test]
fn test_lower() {
    preinterpret_assert_eq!([!lower! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "my_mixedcasestringwhichis  #awesome  -whatdo youthink?");
    preinterpret_assert_eq!([!lower! UPPER], "upper");
    preinterpret_assert_eq!([!lower! lower], "lower");
    preinterpret_assert_eq!([!lower! lower_snake_case], "lower_snake_case");
    preinterpret_assert_eq!([!lower! UPPER_SNAKE_CASE], "upper_snake_case");
    preinterpret_assert_eq!([!lower! lowerCamelCase], "lowercamelcase");
    preinterpret_assert_eq!([!lower! UpperCamelCase], "uppercamelcase");
    preinterpret_assert_eq!([!lower! Capitalized], "capitalized");
    preinterpret_assert_eq!([!lower! "THEY SAID: A quick brown fox jumps over the lazy dog."], "they said: a quick brown fox jumps over the lazy dog.");
    preinterpret_assert_eq!([!lower! "hello_w🌎rld"], "hello_w🌎rld");
    preinterpret_assert_eq!([!lower! "kebab-case"], "kebab-case");
    preinterpret_assert_eq!([!lower! "~~h4xx0rZ <3 1337c0de"], "~~h4xx0rz <3 1337c0de");
    preinterpret_assert_eq!([!lower! PostgreSQLConnection], "postgresqlconnection");
    preinterpret_assert_eq!([!lower! PostgreSqlConnection], "postgresqlconnection");
    preinterpret_assert_eq!([!lower! "U+000A LINE FEED (LF)"], "u+000a line feed (lf)");
    preinterpret_assert_eq!([!lower! "\nThis\r\n is a\tmulti-line\nstring"], "\nthis\r\n is a\tmulti-line\nstring");
    preinterpret_assert_eq!([!lower! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "  lots of _ space and  _whacky |c$ara_cte>>rs|");
    preinterpret_assert_eq!([!lower! "über CöÖl"], "über cööl");
    preinterpret_assert_eq!([!lower! "◌̈ubër Cööl"], "◌̈ubër cööl"); // The ë (and only the e) uses a post-fix combining character
    preinterpret_assert_eq!([!lower! "真是难以置信！"], "真是难以置信！");
}

#[test]
fn test_snake() {
    preinterpret_assert_eq!([!snake! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "my_mixed_case_string_whichis_awesome_whatdo_youthink");
    preinterpret_assert_eq!([!snake! UPPER], "upper");
    preinterpret_assert_eq!([!snake! lower], "lower");
    preinterpret_assert_eq!([!snake! lower_snake_case], "lower_snake_case");
    preinterpret_assert_eq!([!snake! UPPER_SNAKE_CASE], "upper_snake_case");
    preinterpret_assert_eq!([!snake! lowerCamelCase], "lower_camel_case");
    preinterpret_assert_eq!([!snake! UpperCamelCase], "upper_camel_case");
    preinterpret_assert_eq!([!snake! Capitalized], "capitalized");
    preinterpret_assert_eq!([!snake! "THEY SAID: A quick brown fox jumps over the lazy dog."], "they_said_a_quick_brown_fox_jumps_over_the_lazy_dog");
    preinterpret_assert_eq!([!snake! "hello_w🌎rld"], "hello_w_rld");
    preinterpret_assert_eq!([!snake! "kebab-case"], "kebab_case");
    preinterpret_assert_eq!([!snake! "~~h4xx0rZ <3 1337c0de"], "h4xx0r_z_3_1337c0de");
    preinterpret_assert_eq!([!snake! PostgreSQLConnection], "postgre_sql_connection");
    preinterpret_assert_eq!([!snake! PostgreSqlConnection], "postgre_sql_connection");
    preinterpret_assert_eq!([!snake! "U+000A LINE FEED (LF)"], "u_000a_line_feed_lf");
    preinterpret_assert_eq!([!snake! "\nThis\r\n is a\tmulti-line\nstring"], "this_is_a_multi_line_string");
    preinterpret_assert_eq!([!snake! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "lots_of_space_and_whacky_c_ara_cte_rs");
    preinterpret_assert_eq!([!snake! "über CöÖl"], "über_cö_öl");
    preinterpret_assert_eq!([!snake! "◌̈ubër Cööl"], "ube_r_cööl"); // The ë (and only the e) uses a post-fix combining character
    preinterpret_assert_eq!([!snake! "真是难以置信！"], "真是难以置信");
}

#[test]
fn test_upper_snake() {
    preinterpret_assert_eq!([!upper_snake! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "MY_MIXED_CASE_STRING_WHICHIS_AWESOME_WHATDO_YOUTHINK");
    preinterpret_assert_eq!([!upper_snake! UPPER], "UPPER");
    preinterpret_assert_eq!([!upper_snake! lower], "LOWER");
    preinterpret_assert_eq!([!upper_snake! lower_snake_case], "LOWER_SNAKE_CASE");
    preinterpret_assert_eq!([!upper_snake! UPPER_SNAKE_CASE], "UPPER_SNAKE_CASE");
    preinterpret_assert_eq!([!upper_snake! lowerCamelCase], "LOWER_CAMEL_CASE");
    preinterpret_assert_eq!([!upper_snake! UpperCamelCase], "UPPER_CAMEL_CASE");
    preinterpret_assert_eq!([!upper_snake! Capitalized], "CAPITALIZED");
    preinterpret_assert_eq!([!upper_snake! "THEY SAID: A quick brown fox jumps over the lazy dog."], "THEY_SAID_A_QUICK_BROWN_FOX_JUMPS_OVER_THE_LAZY_DOG");
    preinterpret_assert_eq!([!upper_snake! "hello_w🌎rld"], "HELLO_W_RLD");
    preinterpret_assert_eq!([!upper_snake! "kebab-case"], "KEBAB_CASE");
    preinterpret_assert_eq!([!upper_snake! "~~h4xx0rZ <3 1337c0de"], "H4XX0R_Z_3_1337C0DE");
    preinterpret_assert_eq!([!upper_snake! PostgreSQLConnection], "POSTGRE_SQL_CONNECTION");
    preinterpret_assert_eq!([!upper_snake! PostgreSqlConnection], "POSTGRE_SQL_CONNECTION");
    preinterpret_assert_eq!([!upper_snake! "U+000A LINE FEED (LF)"], "U_000A_LINE_FEED_LF");
    preinterpret_assert_eq!([!upper_snake! "\nThis\r\n is a\tmulti-line\nstring"], "THIS_IS_A_MULTI_LINE_STRING");
    preinterpret_assert_eq!([!upper_snake! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "LOTS_OF_SPACE_AND_WHACKY_C_ARA_CTE_RS");
    preinterpret_assert_eq!([!upper_snake! "über CöÖl"], "ÜBER_CÖ_ÖL");
    preinterpret_assert_eq!([!upper_snake! "◌̈ubër Cöol"], "UBE_R_CÖOL"); // The ë (and only the e) uses a post-fix combining character
    preinterpret_assert_eq!([!upper_snake! "真是难以置信！"], "真是难以置信");
}

#[test]
fn test_to_lower_kebab_case() {
    preinterpret_assert_eq!([!kebab! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "my-mixed-case-string-whichis-awesome-whatdo-youthink");
    preinterpret_assert_eq!([!kebab! UPPER], "upper");
    preinterpret_assert_eq!([!kebab! lower], "lower");
    preinterpret_assert_eq!([!kebab! lower_snake_case], "lower-snake-case");
    preinterpret_assert_eq!([!kebab! UPPER_SNAKE_CASE], "upper-snake-case");
    preinterpret_assert_eq!([!kebab! lowerCamelCase], "lower-camel-case");
    preinterpret_assert_eq!([!kebab! UpperCamelCase], "upper-camel-case");
    preinterpret_assert_eq!([!kebab! Capitalized], "capitalized");
    preinterpret_assert_eq!([!kebab! "THEY SAID: A quick brown fox jumps over the lazy dog."], "they-said-a-quick-brown-fox-jumps-over-the-lazy-dog");
    preinterpret_assert_eq!([!kebab! "hello_w🌎rld"], "hello-w-rld");
    preinterpret_assert_eq!([!kebab! "kebab-case"], "kebab-case");
    preinterpret_assert_eq!([!kebab! "~~h4xx0rZ <3 1337c0de"], "h4xx0r-z-3-1337c0de");
    preinterpret_assert_eq!([!kebab! PostgreSQLConnection], "postgre-sql-connection");
    preinterpret_assert_eq!([!kebab! PostgreSqlConnection], "postgre-sql-connection");
    preinterpret_assert_eq!([!kebab! "U+000A LINE FEED (LF)"], "u-000a-line-feed-lf");
    preinterpret_assert_eq!([!kebab! "\nThis\r\n is a\tmulti-line\nstring"], "this-is-a-multi-line-string");
    preinterpret_assert_eq!([!kebab! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "lots-of-space-and-whacky-c-ara-cte-rs");
    preinterpret_assert_eq!([!kebab! "über CöÖl"], "über-cö-öl");
    preinterpret_assert_eq!([!kebab! "◌̈ubër Cööl"], "ube-r-cööl"); // The ë (and only the e) uses a post-fix combining character
    preinterpret_assert_eq!([!kebab! "真是难以置信！"], "真是难以置信");
}

#[test]
fn test_lower_camel() {
    preinterpret_assert_eq!([!lower_camel! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "myMixedCaseStringWhichisAwesomeWhatdoYouthink");
    preinterpret_assert_eq!([!lower_camel! UPPER], "upper");
    preinterpret_assert_eq!([!lower_camel! lower], "lower");
    preinterpret_assert_eq!([!lower_camel! lower_snake_case], "lowerSnakeCase");
    preinterpret_assert_eq!([!lower_camel! UPPER_SNAKE_CASE], "upperSnakeCase");
    preinterpret_assert_eq!([!lower_camel! lowerCamelCase], "lowerCamelCase");
    preinterpret_assert_eq!([!lower_camel! UpperCamelCase], "upperCamelCase");
    preinterpret_assert_eq!([!lower_camel! Capitalized], "capitalized");
    preinterpret_assert_eq!([!lower_camel! "THEY SAID: A quick brown fox jumps over the lazy dog."], "theySaidAQuickBrownFoxJumpsOverTheLazyDog");
    preinterpret_assert_eq!([!lower_camel! "hello_w🌎rld"], "helloWRld");
    preinterpret_assert_eq!([!lower_camel! "kebab-case"], "kebabCase");
    preinterpret_assert_eq!([!lower_camel! "~~h4xx0rZ <3 1337c0de"], "h4xx0rZ31337c0de");
    preinterpret_assert_eq!([!lower_camel! PostgreSQLConnection], "postgreSqlConnection");
    preinterpret_assert_eq!([!lower_camel! PostgreSqlConnection], "postgreSqlConnection");
    preinterpret_assert_eq!([!lower_camel! "U+000A LINE FEED (LF)"], "u000aLineFeedLf");
    preinterpret_assert_eq!([!lower_camel! "\nThis\r\n is a\tmulti-line\nstring"], "thisIsAMultiLineString");
    preinterpret_assert_eq!([!lower_camel! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "lotsOfSpaceAndWhackyCAraCteRs");
    preinterpret_assert_eq!([!lower_camel! "über CöÖl"], "überCöÖl");
    preinterpret_assert_eq!([!lower_camel! "◌̈ubër Cööl"], "ubeRCööl"); // The ë (and only the e) uses a post-fix combining character
    preinterpret_assert_eq!([!lower_camel! "真是难以置信！"], "真是难以置信");
}

#[test]
fn test_camel() {
    preinterpret_assert_eq!([!camel! my_ MixedCase STRING Which is "  #awesome  " - what "do you" think?], "MyMixedCaseStringWhichisAwesomeWhatdoYouthink");
    preinterpret_assert_eq!([!camel! UPPER], "Upper");
    preinterpret_assert_eq!([!camel! lower], "Lower");
    preinterpret_assert_eq!([!camel! lower_snake_case], "LowerSnakeCase");
    preinterpret_assert_eq!([!camel! UPPER_SNAKE_CASE], "UpperSnakeCase");
    preinterpret_assert_eq!([!camel! lowerCamelCase], "LowerCamelCase");
    preinterpret_assert_eq!([!camel! UpperCamelCase], "UpperCamelCase");
    preinterpret_assert_eq!([!camel! Capitalized], "Capitalized");
    preinterpret_assert_eq!([!camel! "THEY SAID: A quick brown fox jumps over the lazy dog."], "TheySaidAQuickBrownFoxJumpsOverTheLazyDog");
    preinterpret_assert_eq!([!camel! "hello_w🌎rld"], "HelloWRld");
    preinterpret_assert_eq!([!camel! "kebab-case"], "KebabCase");
    preinterpret_assert_eq!([!camel! "~~h4xx0rZ <3 1337c0de"], "H4xx0rZ31337c0de");
    preinterpret_assert_eq!([!camel! PostgreSQLConnection], "PostgreSqlConnection");
    preinterpret_assert_eq!([!camel! PostgreSqlConnection], "PostgreSqlConnection");
    preinterpret_assert_eq!([!camel! "U+000A LINE FEED (LF)"], "U000aLineFeedLf");
    preinterpret_assert_eq!([!camel! "\nThis\r\n is a\tmulti-line\nstring"], "ThisIsAMultiLineString");
    preinterpret_assert_eq!([!camel! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "LotsOfSpaceAndWhackyCAraCteRs");
    preinterpret_assert_eq!([!camel! "über CöÖl"], "ÜberCöÖl");
    preinterpret_assert_eq!([!camel! "◌̈ubër Cööl"], "UbeRCööl"); // The ë (and only the e) uses a post-fix combining character
    preinterpret_assert_eq!([!camel! "真是难以置信！"], "真是难以置信");
}

#[test]
fn test_capitalize() {
    preinterpret_assert_eq!([!capitalize! my_ MixedCase STRING Which is "  #awesome  " - what do you think?], "My_MixedCaseSTRINGWhichis  #awesome  -whatdoyouthink?");
    preinterpret_assert_eq!([!capitalize! UPPER], "UPPER");
    preinterpret_assert_eq!([!capitalize! lower], "Lower");
    preinterpret_assert_eq!([!capitalize! lower_snake_case], "Lower_snake_case");
    preinterpret_assert_eq!([!capitalize! UPPER_SNAKE_CASE], "UPPER_SNAKE_CASE");
    preinterpret_assert_eq!([!capitalize! lowerCamelCase], "LowerCamelCase");
    preinterpret_assert_eq!([!capitalize! UpperCamelCase], "UpperCamelCase");
    preinterpret_assert_eq!([!capitalize! Capitalized], "Capitalized");
    preinterpret_assert_eq!([!capitalize! "THEY SAID: A quick brown fox jumps over the lazy dog."], "THEY SAID: A quick brown fox jumps over the lazy dog.");
    preinterpret_assert_eq!([!capitalize! "hello_w🌎rld"], "Hello_w🌎rld");
    preinterpret_assert_eq!([!capitalize! "kebab-case"], "Kebab-case");
    preinterpret_assert_eq!([!capitalize! "~~h4xx0rZ <3 1337c0de"], "~~H4xx0rZ <3 1337c0de");
    preinterpret_assert_eq!([!capitalize! PostgreSQLConnection], "PostgreSQLConnection");
    preinterpret_assert_eq!([!capitalize! PostgreSqlConnection], "PostgreSqlConnection");
    preinterpret_assert_eq!([!capitalize! "U+000A LINE FEED (LF)"], "U+000A LINE FEED (LF)");
    preinterpret_assert_eq!([!capitalize! "\nThis\r\n is a\tmulti-line\nstring"], "\nThis\r\n is a\tmulti-line\nstring");
    preinterpret_assert_eq!([!capitalize! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "  Lots of _ space and  _whacky |c$ara_cte>>rs|");
    preinterpret_assert_eq!([!capitalize! "über CöÖl"], "Über CöÖl");
    preinterpret_assert_eq!([!capitalize! "◌̈ubër Cööl"], "◌̈Ubër Cööl"); // The ë (and only the e) uses a post-fix combining character
    preinterpret_assert_eq!([!capitalize! "真是难以置信！"], "真是难以置信！");
}

#[test]
fn test_decapitalize() {
    preinterpret_assert_eq!([!decapitalize! my_ MixedCase STRING Which is "  #awesome  " - what do you think?], "my_MixedCaseSTRINGWhichis  #awesome  -whatdoyouthink?");
    preinterpret_assert_eq!([!decapitalize! UPPER], "uPPER");
    preinterpret_assert_eq!([!decapitalize! lower], "lower");
    preinterpret_assert_eq!([!decapitalize! lower_snake_case], "lower_snake_case");
    preinterpret_assert_eq!([!decapitalize! UPPER_SNAKE_CASE], "uPPER_SNAKE_CASE");
    preinterpret_assert_eq!([!decapitalize! lowerCamelCase], "lowerCamelCase");
    preinterpret_assert_eq!([!decapitalize! UpperCamelCase], "upperCamelCase");
    preinterpret_assert_eq!([!decapitalize! Capitalized], "capitalized");
    preinterpret_assert_eq!([!decapitalize! "THEY SAID: A quick brown fox jumps over the lazy dog."], "tHEY SAID: A quick brown fox jumps over the lazy dog.");
    preinterpret_assert_eq!([!decapitalize! "hello_w🌎rld"], "hello_w🌎rld");
    preinterpret_assert_eq!([!decapitalize! "kebab-case"], "kebab-case");
    preinterpret_assert_eq!([!decapitalize! "~~h4xx0rZ <3 1337c0de"], "~~h4xx0rZ <3 1337c0de");
    preinterpret_assert_eq!([!decapitalize! PostgreSQLConnection], "postgreSQLConnection");
    preinterpret_assert_eq!([!decapitalize! PostgreSqlConnection], "postgreSqlConnection");
    preinterpret_assert_eq!([!decapitalize! "U+000A LINE FEED (LF)"], "u+000A LINE FEED (LF)");
    preinterpret_assert_eq!([!decapitalize! "\nThis\r\n is a\tmulti-line\nstring"], "\nthis\r\n is a\tmulti-line\nstring");
    preinterpret_assert_eq!([!decapitalize! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "  lots of _ space and  _whacky |c$ara_cte>>rs|");
    preinterpret_assert_eq!([!decapitalize! "über CöÖl"], "über CöÖl");
    preinterpret_assert_eq!([!decapitalize! "◌̈ubër Cööl"], "◌̈ubër Cööl"); // The ë (and only the e) uses a post-fix combining character
    preinterpret_assert_eq!([!decapitalize! "真是难以置信！"], "真是难以置信！");
}

#[test]
fn test_title() {
    preinterpret_assert_eq!([!title! my_ MixedCase STRING Which is "  #awesome  " - what do you think?], "My Mixed Case String Whichis Awesome Whatdoyouthink");
    preinterpret_assert_eq!([!title! UPPER], "Upper");
    preinterpret_assert_eq!([!title! lower], "Lower");
    preinterpret_assert_eq!([!title! lower_snake_case], "Lower Snake Case");
    preinterpret_assert_eq!([!title! UPPER_SNAKE_CASE], "Upper Snake Case");
    preinterpret_assert_eq!([!title! lowerCamelCase], "Lower Camel Case");
    preinterpret_assert_eq!([!title! UpperCamelCase], "Upper Camel Case");
    preinterpret_assert_eq!([!title! Capitalized], "Capitalized");
    preinterpret_assert_eq!([!title! "THEY SAID: A quick brown fox jumps over the lazy dog."], "They Said A Quick Brown Fox Jumps Over The Lazy Dog");
    preinterpret_assert_eq!([!title! "hello_w🌎rld"], "Hello W Rld");
    preinterpret_assert_eq!([!title! "kebab-case"], "Kebab Case");
    preinterpret_assert_eq!([!title! "~~h4xx0rZ <3 1337c0de"], "H4xx0r Z 3 1337c0de");
    preinterpret_assert_eq!([!title! PostgreSQLConnection], "Postgre Sql Connection");
    preinterpret_assert_eq!([!title! PostgreSqlConnection], "Postgre Sql Connection");
    preinterpret_assert_eq!([!title! "U+000A LINE FEED (LF)"], "U 000a Line Feed Lf");
    preinterpret_assert_eq!([!title! "\nThis\r\n is a\tmulti-line\nstring"], "This Is A Multi Line String");
    preinterpret_assert_eq!([!title! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "Lots Of Space And Whacky C Ara Cte Rs");
    preinterpret_assert_eq!([!title! "über CöÖl"], "Über Cö Öl");
    preinterpret_assert_eq!([!title! "◌̈ubër Cööl"], "Ube R Cööl"); // The ë (and only the e) uses a post-fix combining character
    preinterpret_assert_eq!([!title! "真是难以置信！"], "真是难以置信");
}

#[test]
fn test_insert_spaces() {
    preinterpret_assert_eq!([!insert_spaces! my_ MixedCase STRING Which is "  #awesome  " - what do you think?], "my Mixed Case STRING Whichis awesome whatdoyouthink");
    preinterpret_assert_eq!([!insert_spaces! UPPER], "UPPER");
    preinterpret_assert_eq!([!insert_spaces! lower], "lower");
    preinterpret_assert_eq!([!insert_spaces! lower_snake_case], "lower snake case");
    preinterpret_assert_eq!([!insert_spaces! UPPER_SNAKE_CASE], "UPPER SNAKE CASE");
    preinterpret_assert_eq!([!insert_spaces! lowerCamelCase], "lower Camel Case");
    preinterpret_assert_eq!([!insert_spaces! UpperCamelCase], "Upper Camel Case");
    preinterpret_assert_eq!([!insert_spaces! Capitalized], "Capitalized");
    preinterpret_assert_eq!([!insert_spaces! "THEY SAID: A quick brown fox jumps over the lazy dog."], "THEY SAID A quick brown fox jumps over the lazy dog");
    preinterpret_assert_eq!([!insert_spaces! "hello_w🌎rld"], "hello w rld");
    preinterpret_assert_eq!([!insert_spaces! "kebab-case"], "kebab case");
    preinterpret_assert_eq!([!insert_spaces! "~~h4xx0rZ <3 1337c0de"], "h4xx0r Z 3 1337c0de");
    preinterpret_assert_eq!([!insert_spaces! PostgreSQLConnection], "Postgre SQL Connection");
    preinterpret_assert_eq!([!insert_spaces! PostgreSqlConnection], "Postgre Sql Connection");
    preinterpret_assert_eq!([!insert_spaces! "U+000A LINE FEED (LF)"], "U 000A LINE FEED LF");
    preinterpret_assert_eq!([!insert_spaces! "\nThis\r\n is a\tmulti-line\nstring"], "This is a multi line string");
    preinterpret_assert_eq!([!insert_spaces! "  lots of _ space and  _whacky |c$ara_cte>>rs|"], "lots of space and whacky c ara cte rs");
    preinterpret_assert_eq!([!insert_spaces! "über CöÖl"], "über Cö Öl");
    preinterpret_assert_eq!([!insert_spaces! "◌̈ubër Cööl"], "ube r Cööl"); // The ë (and only the e) uses a post-fix combining character
    preinterpret_assert_eq!([!insert_spaces! "真是难以置信！"], "真是难以置信");
}
