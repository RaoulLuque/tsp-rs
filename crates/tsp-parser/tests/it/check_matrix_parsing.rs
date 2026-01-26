use std::{
    self,
    fs::File,
    io::{BufRead, BufReader},
};

use tsp_core::instance::{
    TSPSymInstance,
    distance::Distance,
    matrix::{SquareMatrix, TriangularMatrix},
    node::Node,
};
use tsp_macros::test_fn_on_all_instances;

fn check_input_file_against_golden_file(instance_path: &str) {
    let golden_file_path = format!(
        "tests/test_assets/symmetric_matrices/{}.txt",
        instance_path
            .split('/')
            .last()
            .unwrap()
            .strip_suffix(".tsp")
            .unwrap()
    );
    if let Ok(file) = File::open(&golden_file_path) {
        let input_instance_sym: TSPSymInstance<TriangularMatrix<Distance>> =
            tsp_parser::parse_tsp_instance(instance_path)
                .expect("Symmetric parsing should succeed");
        // let input_instance_matrix: TSPSymInstance<SquareMatrix<Distance>> =
        //     tsp_parser::parse_tsp_instance(instance_path).expect("Matrix parsing should
        // succeed");
        let golden_distance_data = BufReader::new(file)
            .lines()
            .map(|line| {
                let line = line.unwrap();
                line.split(",")
                    .map(|entry| Distance(entry.trim().parse::<i32>().unwrap()))
                    .collect::<Vec<Distance>>()
                    .into_iter()
            })
            .flatten()
            .collect::<Vec<Distance>>();

        assert_eq!(
            golden_distance_data.len(),
            input_instance_sym.raw_distances().len()
        );
        println!(
            "Symmetric Matrix: \n{}",
            input_instance_sym.distance_matrix()
        );
        for (i, &distance) in golden_distance_data.iter().enumerate() {
            assert_eq!(
                distance,
                input_instance_sym.raw_distances()[i],
                "Distance data mismatch at index {} with values {:?} (expected) vs {:?} (actual)",
                i,
                distance,
                input_instance_sym.raw_distances()[i]
            );
        }
        // println!("Matrix: \n{}", input_instance_matrix.distance_matrix());
        // assert_eq!(input_instance_sym.raw_distances(), golden_distance_data);
        // for row in 0..input_instance_matrix.distance_matrix().dimension() {
        //     for col in 0..input_instance_matrix.distance_matrix().dimension() {
        //         assert_eq!(
        //             input_instance_matrix
        //                 .distance_matrix()
        //                 .get_data(Node(row), Node(col)),
        //             input_instance_sym
        //                 .distance_matrix()
        //                 .get_data(Node(row), Node(col)),
        //             "Distance matrix mismatch at position ({}, {}) with values {:?} (symmetric) \
        //              vs {:?} (matrix)",
        //             row,
        //             col,
        //             input_instance_sym
        //                 .distance_matrix()
        //                 .get_data(Node(row), Node(col)),
        //             input_instance_matrix
        //                 .distance_matrix()
        //                 .get_data(Node(row), Node(col))
        //         );
        //     }
        // }
    }
}

// This doesn't actually need to be run on all instances, but might as well reuse the macro since
// we have it.
test_fn_on_all_instances!(
    check_input_file_against_golden_file,
    check_matrix_parsing,
    0,
    550
);
