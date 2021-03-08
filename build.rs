// From: https://docs.rs/crate/gdcm_conv/0.1.0/s
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn build() {
    // run GDCM cmake
    let mut cfg = cmake::Config::new("GDCM");

    let dst = cfg
        .define("GDCM_BUILD_TESTING", "OFF")
        .define("GDCM_DOCUMENTATION", "OFF")
        .define("GDCM_BUILD_EXAMPLES", "OFF")
        .define("GDCM_BUILD_DOCBOOK_MANPAGES", "OFF")
        .cflag("-fPIC")
        .cxxflag("-std=c++11")
        .uses_cxx11()
        .build_arg("-j8")
        .build();

    // set GDCM include path
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let include_dir = out_path.join("include").join("gdcm-3.1");

    // create library
    cc::Build::new()
        .file("gdcm_wrapper.cc")
        .cpp(true)
        .flag("-fPIC")
        .flag("-std=c++11")
        .include(include_dir)
        .compile("gdcm_wrapper");

    // set libs paths
    println!("cargo:rustc-link-search={}", dst.join("lib").display());
    println!("cargo:rustc-link-search={}", dst.display());

    // set libs
    println!("cargo:rustc-link-lib=static=gdcm_wrapper");

    // gdcm libs
    println!("cargo:rustc-link-lib=static=gdcmMSFF");
    println!("cargo:rustc-link-lib=static=gdcmcharls");
    println!("cargo:rustc-link-lib=static=gdcmCommon");
    println!("cargo:rustc-link-lib=static=gdcmDICT");
    println!("cargo:rustc-link-lib=static=gdcmDSED");
    println!("cargo:rustc-link-lib=static=gdcmIOD");
    println!("cargo:rustc-link-lib=static=gdcmexpat");
    println!("cargo:rustc-link-lib=static=gdcmjpeg12");
    println!("cargo:rustc-link-lib=static=gdcmjpeg16");
    println!("cargo:rustc-link-lib=static=gdcmjpeg8");
    println!("cargo:rustc-link-lib=static=gdcmopenjp2");
    println!("cargo:rustc-link-lib=static=gdcmuuid");
    println!("cargo:rustc-link-lib=static=gdcmMEXD");
    println!("cargo:rustc-link-lib=static=gdcmzlib");

    // FIXME: OSX ONLY
    println!("Building for {}", env::consts::OS);
    match env::consts::OS {
        "macos" => {
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!(
                "cargo:rustc-link-search=framework={}",
                "/System/Library/Frameworks"
            );
        }
        _ => {
            // Probably not supported
        }
    };
}

fn main() {
    // re-build if files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=gdcm_wrapper.cc");
    println!("cargo:rerun-if-changed=wrapper.h");

    // unset DESTDIR envar to avoid others libs destinations
    env::remove_var("DESTDIR");

    // update git
    if !Path::new("GDCM/.git").exists() {
        let _ = Command::new("git")
            .args(&["submodule", "update", "--init"])
            .status();
    }

    build();
}
