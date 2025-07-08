use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    cc::Build::new()
        .file("boot.s")
        .flag("-march=rv64gc")
        .flag("-mabi=lp64d")
        .flag("-nostdlib")
        .flag("-nostartfiles")
        .compile("boot");

    println!("cargo:rerun-if-changed=boot.s");
}
