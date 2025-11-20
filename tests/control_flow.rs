#![allow(clippy::identity_op)] // https://github.com/rust-lang/rust-clippy/issues/13924

#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_control_flow_compilation_failures() {
    if !should_run_ui_tests() {
        // Some of the outputs are different on nightly, so don't test these
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/control_flow/**/*.rs");
}

#[test]
fn test_if() {
    assert_eq!(
        run! {
            if (1 == 2) { "YES" } else { "NO" }
        },
        "NO"
    );
    assert_eq!(
        run! {
            let x = 1 == 2;
            if x { "YES" } else { "NO" }
        },
        "NO"
    );
    assert_eq!(
        run! {
            let x = 1;
            let y = 2;
            if x == y { "YES" } else { "NO" }
        },
        "NO"
    );
    assert_eq!(
        run! {
            %[0 #(if true { %[+ 1] })]
        },
        1
    );
    assert_eq!(
        stream! {
            0
            #(if false { %[+ 1] })
        },
        0
    );
    assert_eq!(
        run! {
            if false {
                1
            } else if false {
                2
            } else if true {
                3
            } else {
                4
            }
        },
        3
    );
}

#[test]
fn test_while() {
    assert_eq!(
        run! {
            let x = 0;
            while x < 5 {
                x += 1;
            }
            x
        },
        5
    );
}

#[test]
fn test_loop_continue_and_break() {
    assert_eq!(
        run! {
            let x = 0;
            loop {
                x += 1;
                if x >= 10 {
                    break;
                }
            }
            x
        },
        10
    );
    assert_eq!(
        run! {
            let arr = [];
            for x in 65..75 {
                if x % 2 == 0 {
                    continue;
                }
                arr.push(x as u8 as char);
            }
            arr.to_string()
        },
        "ACEGI"
    );
}

#[test]
fn test_for() {
    assert_eq!(
        run! {
            let arr = [];
            for x in 65..70 {
                arr.push(x as u8 as char);
            }
            arr.to_string()
        },
        "ABCDE"
    );
    assert_eq!(
        run! {
            // A stream is iterated token-tree by token-tree
            // So we can match each value with a stream pattern matching each `(X,)`
            let arr = [];
            for %[(@(#x = @IDENT),)] in %[(a,) (b,) (c,)] {
                if x.to_string() == "c" {
                    break;
                }
                arr.push(x.to_string());
            }
            emit arr.to_string();
        },
        "ab"
    );
}

#[test]
fn test_attempt() {
    let x = run! {
        attempt {
            {} => { 1 }
            {} => { 2 }
        }
    };
    assert_eq!(x, 1);
    run! {
        let x = 0;
        let output = attempt {
            { %[_].assert_eq(x, 1) } => { 1 }
            { let x = 2; } => { x }
        };
        %[_].assert_eq(output, 2);
    }
    // Mutating a variable defined inside the arm works.
    // (Not being able to mutate parent scope variables is tested as a compilation failure.)
    run! {
        let value = attempt {
            { let x = 0; x += 1; } => { x }
        };
        %[_].assert_eq(value, 1);
    }
    run! {
        let x = 0;
        attempt {
            { } => { x += 1; }
        };
        %[_].assert_eq(x, 1);
    }
    run! {
        let y = 3;
        let value = attempt {
            { let x = y + 1; } => { x }
        };
        %[_].assert_eq(value, 4);
    }
    run! {
        let value = attempt {
            {
                let y = 1;
                attempt {
                    {} => { y += 1; }
                }
            } => { y }
        };
        %[_].assert_eq(value, 2);
    }
    // Control flow interrupts can be inside attempt arm LHS.
    run! {
        loop {
            attempt {
                { break; } => { None }
            }
            %[_].error("Should be unreachable");
        }
    }
    // Control flow interrupts can be inside attempt arm RHS.
    run! {
        loop {
            attempt {
                {} => { break; }
            }
            %[_].error("Should be unreachable");
        }
    }
    run! {
        let value = attempt {
            { revert; } => { None }
            { } => 1,
        };
        %[_].assert_eq(value, 1);
    }
    run!(
        let value = attempt {
            {
                // This revert propagates to the LHS of the outer attempt
                attempt {
                    { } => { revert; }
                    { } => { None }
                }
            } => { 1 }
            { } => { 2 }
        };
        %[_].assert_eq(value, 2);
    );
}

#[test]
fn test_attempt_guard_clauses() {
    run! {
        let output = attempt {
            { } if false => 1,
            { } => 2,
        };
        %[_].assert_eq(output, 2);
    }
    run! {
        let output = attempt {
            { let x = 4; } if { x += 1; true } => { x }
            { } => { 2 }
        };
        %[_].assert_eq(output, 5);
    }
    run! {
        let output = attempt {
            { let x = 2; } if x >= 3 => x,
            { let x = 5; } if x >= 3 => x,
        };
        %[_].assert_eq(output, 5);
    }
}

#[test]
fn test_emit_statement() {
    assert_eq!(
        run! {
            let s = "Hello";
            emit s;
        },
        "Hello"
    );

    assert_eq!(
        stream! {
            [#{
                for i in 1..=5 {
                    emit i;
                    emit %[,];
                }
            }]
        },
        [1, 2, 3, 4, 5]
    );

    run! {
        emit %[
            fn my_add(a: i32, b: i32) -> i32 {
                a + b
            }
        ];
        emit %[
            fn my_sub(a: i32, b: i32) -> i32 {
                a - b
            }
        ];
        // Final return is also emitted
        %[
            fn my_mul(a: i32, b: i32) -> i32 {
                a * b
            }
        ]
    };
    assert!(my_add(5, 3) == 8);
    assert!(my_sub(5, 3) == 2);
    assert!(my_mul(5, 3) == 15);

    // Internal emits inside revertible segments are OK
    assert_eq!(
        run! {
            attempt {
                {
                    let x = %[#{ emit 1; }];
                } => { emit %[#x + #x]; }
            }
        },
        2
    );
}

#[test]
fn test_break_with_value() {
    // Simple break with value from loop
    assert_eq!(
        run! {
            let result = loop {
                break 42;
            };
            result
        },
        42
    );

    // Break with computed value
    assert_eq!(
        run! {
            let result = loop {
                let x = 10;
                let y = 5;
                break x + y;
            };
            result
        },
        15
    );

    // While loop with break value
    assert_eq!(
        run! {
            let x = 0;
            let result = while x < 10 {
                x += 1;
                if x == 5 {
                    break x * 2;
                }
            };
            result
        },
        10
    );

    // For loop with break value
    assert_eq!(
        run! {
            let result = for i in 1..=10 {
                if i == 7 {
                    break i * i;
                }
            };
            result
        },
        49
    );
}

#[test]
fn test_labeled_loops() {
    // Labeled loop with break
    assert_eq!(
        run! {
            let result = 'outer: loop {
                break 'outer 100;
            };
            result
        },
        100
    );

    // Labeled while loop
    assert_eq!(
        run! {
            let x = 0;
            'counting: while x < 10 {
                x += 1;
                if x == 3 {
                    break 'counting x;
                }
            }
        },
        3
    );

    // Labeled for loop
    assert_eq!(
        run! {
            'summing: for i in 1..=5 {
                if i == 3 {
                    break 'summing i * 10;
                }
            }
        },
        30
    );
}

#[test]
fn test_nested_loops_with_labels() {
    // Break outer loop from inner loop
    assert_eq!(
        run! {
            let result = 'outer: loop {
                for i in 1..=5 {
                    if i == 3 {
                        break 'outer i + 100;
                    }
                }
                break 'outer 0;
            };
            result
        },
        103
    );

    // Continue outer loop from inner loop
    assert_eq!(
        run! {
            let count = 0;
            'outer: for i in 1..=3 {
                for j in 1..=3 {
                    count += 1;
                    if j == 2 {
                        continue 'outer;
                    }
                }
            }
            count
        },
        6  // Should count: (1,1), (1,2), (2,1), (2,2), (3,1), (3,2)
    );

    // Complex nested example
    assert_eq!(
        run! {
            let result = 'outer: loop {
                'middle: for i in 1..=3 {
                    'inner: for j in 1..=3 {
                        if i == 2 && j == 2 {
                            break 'outer i * 10 + j;
                        }
                    }
                }
                break 'outer 0;
            };
            result
        },
        22
    );
}

#[test]
fn test_labeled_blocks() {
    // Simple labeled block with break
    assert_eq!(
        run! {
            let result = 'block: {
                let x = 5;
                if x > 3 {
                    break 'block x * 2;
                }
                x
            };
            result
        },
        10
    );

    // Labeled block with conditional break
    assert_eq!(
        run! {
            let result = 'compute: {
                let value = 10;
                if value < 5 {
                    break 'compute 0;
                } else if value < 15 {
                    break 'compute value * 2;
                }
                value * 3
            };
            result
        },
        20
    );

    // Nested labeled blocks
    assert_eq!(
        run! {
            let result = 'outer: {
                let x = 5;
                'inner: {
                    if x > 3 {
                        break 'outer x + 100;
                    }
                    break 'inner x;
                };
                x * 2
            };
            result
        },
        105
    );

    // Labeled block with loop inside
    assert_eq!(
        run! {
            'block: {
                for i in 1..=5 {
                    if i == 3 {
                        break 'block i * 10;
                    }
                }
                0
            }
        },
        30
    );
}
