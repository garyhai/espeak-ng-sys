#![allow(clippy::uninlined_format_args)]

extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    // Link C++ standard library
    if let Some(cpp_stdlib) = get_cpp_link_stdlib(&target) {
        println!("cargo:rustc-link-lib=dylib={}", cpp_stdlib);
    }

    println!("cargo:rerun-if-changed=wrapper.h");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    let bindings = bindgen::Builder::default()
        .header("include/wrapper.h")
        .clang_arg("-I./espeak-ng/src/include/espeak-ng")
        .clang_arg("-I./espeak-ng/src/libespeak-ng")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate();

    match bindings {
        Ok(b) => {
            let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
            b.write_to_file(out_path.join("bindings.rs"))
                .expect("Couldn't write bindings!");
        }
        Err(e) => {
            println!("cargo:warning=Unable to generate bindings: {}", e);
            println!("cargo:warning=Using bundled bindings.rs, which may be out of date");
            // copy src/bindings.rs to OUT_DIR
            std::fs::copy("src/bindings.rs", out.join("bindings.rs"))
                .expect("Unable to copy bindings.rs");
        }
    }

    // stop if we're on docs.rs
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    let files = [
        "espeak-ng/src/libespeak-ng/common.c",
        "espeak-ng/src/libespeak-ng/mnemonics.c",
        "espeak-ng/src/libespeak-ng/error.c",
        "espeak-ng/src/libespeak-ng/ieee80.c",
        "espeak-ng/src/libespeak-ng/compiledata.c",
        "espeak-ng/src/libespeak-ng/compiledict.c",
        "espeak-ng/src/libespeak-ng/dictionary.c",
        "espeak-ng/src/libespeak-ng/encoding.c",
        "espeak-ng/src/libespeak-ng/intonation.c",
        "espeak-ng/src/libespeak-ng/langopts.c",
        "espeak-ng/src/libespeak-ng/numbers.c",
        "espeak-ng/src/libespeak-ng/phoneme.c",
        "espeak-ng/src/libespeak-ng/phonemelist.c",
        "espeak-ng/src/libespeak-ng/readclause.c",
        "espeak-ng/src/libespeak-ng/setlengths.c",
        "espeak-ng/src/libespeak-ng/soundicon.c",
        "espeak-ng/src/libespeak-ng/spect.c",
        "espeak-ng/src/libespeak-ng/ssml.c",
        "espeak-ng/src/libespeak-ng/synthdata.c",
        "espeak-ng/src/libespeak-ng/synthesize.c",
        "espeak-ng/src/libespeak-ng/tr_languages.c",
        "espeak-ng/src/libespeak-ng/translate.c",
        "espeak-ng/src/libespeak-ng/translateword.c",
        "espeak-ng/src/libespeak-ng/voices.c",
        "espeak-ng/src/libespeak-ng/wavegen.c",
        "espeak-ng/src/libespeak-ng/speech.c",
        "espeak-ng/src/libespeak-ng/espeak_api.c",
        "espeak-ng/src/ucd-tools/src/case.c",
        "espeak-ng/src/ucd-tools/src/categories.c",
        "espeak-ng/src/ucd-tools/src/ctype.c",
        "espeak-ng/src/ucd-tools/src/proplist.c",
        "espeak-ng/src/ucd-tools/src/scripts.c",
        "espeak-ng/src/ucd-tools/src/tostring.c",
    ];
    let includes = [
        "./include",
        "espeak-ng/src/ucd-tools/src/include",
        "espeak-ng/src/include",
        "espeak-ng/src/include/compat",
        "espeak-ng/src/include/espeak-ng",
    ];
    cc::Build::new()
        .files(files)
        .includes(includes)
        .target(&target)
        .static_flag(true)
        .flag("-Wno-error=implicit-function-declaration")
        .flag("-w")
        .compile("espeak-ng");

    if env::var("TARGET").unwrap().contains("window") {
        println!(
            "cargo:rustc-link-search={}",
            out.join("build").join("Release").display()
        );
    } else {
        println!("cargo:rustc-link-search={}", out.join("build").display());
    }
}

// From https://github.com/alexcrichton/cc-rs/blob/fba7feded71ee4f63cfe885673ead6d7b4f2f454/src/lib.rs#L2462
fn get_cpp_link_stdlib(target: &str) -> Option<&'static str> {
    if target.contains("msvc") {
        None
    } else if target.contains("apple") || target.contains("freebsd") || target.contains("openbsd") {
        Some("c++")
    } else if target.contains("android") {
        Some("c++_shared")
    } else {
        Some("stdc++")
    }
}
