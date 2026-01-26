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
        .join("instances")
        .join("tsplib_symmetric");
    let output_directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
        .join("test_assets")
        .join("distances_square_matrix");

    println!("Reading instances from {:?}", instance_directory);
    println!("Writing golden parsing files to {:?}", output_directory);

    let instance_dir = std::fs::read_dir(&instance_directory).unwrap();

    for instance_path in instance_dir {
        let instance_path = instance_path.unwrap().path();
        if instance_path.extension().and_then(|s| s.to_str()) != Some("tsp") {
            continue;
        }

        let instance_name = instance_path.file_stem().and_then(|s| s.to_str()).unwrap();

        let output_file_path = output_directory.join(format!("{}.txt", instance_name));

        let size = instance_name[instance_name
            .chars()
            .enumerate()
            .find(|(_, c)| c.is_ascii_digit())
            .unwrap()
            .0..]
            .parse::<usize>()
            .unwrap();
        if size > 100 {
            continue;
        }

        let instance: TSPSymInstance<SquareMatrix<Distance>> =
            parse_tsp_instance(instance_path.to_str().unwrap()).expect("Parsing should succeed");

        let mut output_file =
            File::create(&output_file_path).expect("Should be able to create output file");

        let distances_str = instance.distance_matrix().to_string();

        write!(output_file, "{}", distances_str).expect("Should be able to write to output file");
    }
}
