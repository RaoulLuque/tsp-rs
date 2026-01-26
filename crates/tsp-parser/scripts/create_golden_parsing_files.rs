#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[dependencies]
tsp-parser = { version = "0.1", path = "../" }
tsp-core = { version = "0.1", path = "../../tsp-core" }
---

use std::{fs::File, io::Write};

use tsp_core::instance::{TSPSymInstance, distance::Distance, matrix::SquareMatrix};
use tsp_parser::parse_tsp_instance;

fn main() {
    let instance_directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("instances");
    let output_directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
        .join("test_assets")
        .join("distances_square_matrix");

    println!("Reading instances from {:?}", instance_directory);
    println!("Writing golden parsing files to {:?}", output_directory);

    // Iterate over subdirectories in instances/
    let instances_contents = std::fs::read_dir(&instance_directory).unwrap();
    for entry in instances_contents.flatten() {
        let subdir_path = entry.path();
        let subdir_name = subdir_path
            .file_name()
            .and_then(|n: &std::ffi::OsStr| n.to_str())
            .unwrap_or("");

        // Iterate over .tsp files in current subdirectory
        let subdir_contents = std::fs::read_dir(&subdir_path).unwrap();
        for file in subdir_contents.flatten() {
            let instance_path = file.path();
            if instance_path
                .extension()
                .and_then(|s: &std::ffi::OsStr| s.to_str())
                != Some("tsp")
            {
                continue;
            }

            let instance_name = instance_path
                .file_stem()
                .and_then(|s: &std::ffi::OsStr| s.to_str())
                .unwrap();

            let output_file_path = output_directory.join(format!("{}.txt", instance_name));

            let size = instance_name[instance_name
                .chars()
                .enumerate()
                .find(|(_, c): &(usize, char)| c.is_ascii_digit())
                .unwrap()
                .0..]
                .parse::<usize>()
                .unwrap();
            if size > 100 {
                continue;
            }

            let instance: TSPSymInstance<SquareMatrix<Distance>> =
                parse_tsp_instance(instance_path.to_str().unwrap())
                    .expect("Parsing should succeed");

            let mut output_file =
                File::create(&output_file_path).expect("Should be able to create output file");

            let distances_str = instance.distance_matrix().to_string();

            write!(output_file, "{}", distances_str)
                .expect("Should be able to write to output file");
        }
    }
}
