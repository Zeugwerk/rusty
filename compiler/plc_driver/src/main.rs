use std::panic;

fn main() {
    // let thread = std::thread::spawn(move || {
    //     // std::process::Command::new("./demo.st.out").spawn().unwrap();
    //     panic!()
    // });

    // if thread.join().is_err() {
    //     println!("Working, wubwub");
    // }

    let result = panic::catch_unwind(|| {
        panic!("test");
    });

    if result.is_err() {
        println!("Test");
    }

    // //Initialize the logging
    // env_logger::init();
    // let args: Vec<String> = env::args().collect();
    // if let Err(err) = plc_driver::compile(&args) {
    //     err.exit()
    // }
}
