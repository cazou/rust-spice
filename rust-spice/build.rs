/*
 * Copyright Contributors to the tardis project
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use std::{env, fs};
use std::collections::HashSet;
use std::path::PathBuf;

use cc;
use bindgen;

//This is used to avoid double declarations of FP_NAN and such
#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}

fn has_ext<T>(entry: &Result<fs::DirEntry, T>, ext: &str) -> bool {
    if let Ok(e) = entry {
        return match e.file_name().to_str() {
            Some(s) => s.ends_with(ext),
            None => false,
        }
    }

    false
}

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=src/c/");
    println!("cargo:rerun-if-changed=src/includes/");

    //TODO: return an error here
    let entries = match fs::read_dir("/home/detlev/Sources/tardis/cspice/src/c/") {
        Ok(e) => e.filter(|res| has_ext(res, ".c")),
        Err(_) => return,
    };

    let mut v = vec![];

    for e in entries {
        v.push(e.unwrap().path());
    }

    // Use the `cc` crate to build a C file and statically link it.
    cc::Build::new()
        .files(v)
        .include("src/includes")
        .flag("-Wno-dangling-else")
        .compile("libspice.a");

    let ignored_macros = IgnoreMacros(
            vec![
                "FP_INFINITE".into(),
                "FP_NAN".into(),
                "FP_NORMAL".into(),
                "FP_SUBNORMAL".into(),
                "FP_ZERO".into(),
                "IPPORT_RESERVED".into(),
            ]
            .into_iter()
            .collect(),
        );

    let bindings = bindgen::Builder::default()
        .header("src/includes/spice.h")
        .parse_callbacks(Box::new(ignored_macros))
        .generate()
        .expect("Failed to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("c_spice.rs"))
        .expect("Failed to write bindings");
}
