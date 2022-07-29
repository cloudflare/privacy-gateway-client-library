// Copyright (c) 2022 Cloudflare, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use cbindgen::Config;
use std::env;

fn main() {
    // cbindgen crashes on stable release due to macro expansion
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let config = Config {
        language: cbindgen::Language::C,
        documentation_style: cbindgen::DocumentationStyle::C99,
        cpp_compat: true,
        usize_is_size_t: true,
        parse: cbindgen::ParseConfig {
            parse_deps: false,
            include: Some(vec!["apprelay".to_owned()]),
            ..Default::default()
        },
        ..Default::default()
    };

    cbindgen::generate_with_config(&crate_dir, config)
        .expect("Could not generate header")
        .write_to_file("apprelay.h");
}
