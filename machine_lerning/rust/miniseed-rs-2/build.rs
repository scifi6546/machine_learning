fn main() {
    if cfg!(target_os = "windows") {};
    println!(
        "cargo:rustc-link-search=/home/nick/machine_learning/pure_rust/miniseed-rs-2/libmseed-3.2.3/"
    );
    println!("cargo:rustc-link-lib=static=mseed");
    println!(
        "cargo:rustc-link-search=/home/nick/machine_learning/pure_rust/miniseed-rs-2/output/lib/"
    );
}
