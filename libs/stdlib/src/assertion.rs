#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn ASSERT_EQ(a: i32, b: i32) -> i32 {
    if a != b {
        // eprintln!("Assert failed, {a} != {b}");
        // std::process::exit(-1);
        panic!("Assert failed, {a} != {b}");
    }

    // std::process::exit(0);
    0
}
