fn main() {
    if let Err(error) = rocksoul::load_environment() {
        eprintln!("error loading .env: {error:#}");
        std::process::exit(1);
    }

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("error initializing async runtime: {error}");
            std::process::exit(1);
        }
    };

    if let Err(error) = runtime.block_on(rocksoul::run()) {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
