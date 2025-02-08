use std::{env, fs, path::PathBuf};

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out.display());

    fs::copy("./ld/link.x", out.join("link.x")).unwrap();

    println!("cargo:rustc-link-arg=-Tlink.x");
}
