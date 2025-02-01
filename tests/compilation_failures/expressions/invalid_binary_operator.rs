use preinterpret::*;

fn main() {
    let _ = preinterpret! {
        // The error message here isn't as good as it could be,
        // because we end the Expression when we detect an invalid extension.
        // This is useful to allow e.g. [!if! true == false { ... }]
        // But in future, perhaps we can use different parse modes for these cases.
        [!evaluate! 10 _ 10]
    };
}
