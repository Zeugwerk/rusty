#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn ASSERT_EQ(a: i32, b: i32) -> i32 {
    if a != b {
        panic!("Assert failed, {a} != {b}");
    }

    0
}