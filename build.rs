fn main() {
    println!("cargo:rerun-if-env-changed=ROCKSOUL_GIT_SHA");
    let revision = std::env::var("ROCKSOUL_GIT_SHA").unwrap_or_else(|_| "unknown".to_owned());
    println!("cargo:rustc-env=ROCKSOUL_GIT_SHA={revision}");
}
