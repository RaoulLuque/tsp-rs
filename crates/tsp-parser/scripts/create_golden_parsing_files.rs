#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[dependencies]
tsp-parser = { version = "0.1", path = "../" }
---

fn main() {
    let instance_directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("instances")
        .join("tsplib_symmetric");
    let output_directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
        .join("test_assets")
        .join("distances_square_matrix");
    println!("Creating golden parsing files in {:?}", output_directory);
}
