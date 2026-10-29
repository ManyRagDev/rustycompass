pub fn rust_leaf(value: i32) -> i32 {
    value + 2 // DEV_RUST_LITERAL
}
pub fn rust_caller(value: i32) -> i32 {
    rust_leaf(value)
}
