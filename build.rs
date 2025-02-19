use std::{env, fs, path::PathBuf};

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out.display());

    #[cfg(feature = "esp32c2")]
    fs::copy("./ld/link_esp32c2.x", out.join("link.x")).unwrap();

    #[cfg(feature = "esp32c3")]
    fs::copy("./ld/link_esp32c3.x", out.join("link.x")).unwrap();

    #[cfg(feature = "esp32c6")]
    fs::copy("./ld/link_esp32c6.x", out.join("link.x")).unwrap();

    #[cfg(feature = "esp32h2")]
    fs::copy("./ld/link_esp32h2.x", out.join("link.x")).unwrap();

    #[cfg(feature = "esp32")]
    fs::copy("./ld/link_esp32.x", out.join("link.x")).unwrap();

    #[cfg(feature = "esp32s2")]
    fs::copy("./ld/link_esp32s2.x", out.join("link.x")).unwrap();

    #[cfg(feature = "esp32s3")]
    fs::copy("./ld/link_esp32s3.x", out.join("link.x")).unwrap();

    println!("cargo:rustc-link-arg=-Tlink.x");
}
