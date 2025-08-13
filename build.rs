fn main() {
    cc::Build::new()
        .file("boot.s")
        .file("trap.s")
        .flag("-march=rv64gc")
        .flag("-mabi=lp64d")
        .flag("-nostdlib")
        .flag("-nostartfiles")
        .compile("boot");

    println!("cargo:rerun-if-changed=boot.s");
    println!("cargo:rerun-if-changed=trap.s");
}
