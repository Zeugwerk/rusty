use std::panic;
fn main() {
    // let thread = std::thread::spawn(move || {
    //     panic::set_hook(Box::new(|info| {
    //         println!("{}", info.to_string());
    //         println!("hi 2");
    //         // do nothing
    //     }));

    //     let catch = std::panic::catch_unwind(|| {
    //         std::process::Command::new("./demo.st.out").spawn().unwrap();
    //     });
    //     println!("hi");
    //     // extern "C" panic -> UB
    //     // extern "C-unwind" -> unstable
    // });

    // let _ = thread.join();

    // let catch = std::panic::catch_unwind(|| {
    //     dbg!(std::process::Command::new("./demo.st.out").spawn().unwrap());
    // });
    // println!("{}", catch.is_err());

    // if thread.join().is_err() {
    //     println!("Working, wubwub");
    // }
    // let thread = std::thread::spawn(move || {
    //     // std::process::Command::new("./demo.st.out").spawn().unwrap();
    //     panic!()
    // });

    // if thread.join().is_err() {
    //     println!("Working, wubwub");
    // }

    // let res = dbg!(std::process::Command::new("./demo.st.out").output().unwrap());
    // println!("{} \nExit with: {}", String::from_utf8_lossy(&res.stderr), res.status);

    // Initialize the logging
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    if let Err(err) = plc_driver::compile(&args) {
        err.exit()
    }
}
