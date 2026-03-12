use std::env;

fn main() {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    println!("cargo:rustc-link-arg=-L{}", dir);
    println!("cargo:rustc-link-arg=-T{}/link.ld", dir);
}
