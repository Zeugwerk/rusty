fn main() {
    // Initialize the logging
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    if let Err(err) = plc_driver::compile(&args) {
        err.exit()
    }
}
