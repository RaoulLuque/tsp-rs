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
    let Some(golden_distance_data) = parse_golden_file_square_matrix(instance_path) else {
        // If there's no golden file, we can't check anything.
        return;
    };

    let parsed_triangular_matrix: TSPSymInstance<TriangularMatrix<Distance>> =
        tsp_parser::parse_tsp_instance(instance_path).expect("Symmetric parsing should succeed");
    let parsed_square_matrix: TSPSymInstance<SquareMatrix<Distance>> =
        tsp_parser::parse_tsp_instance(instance_path).expect(
            "Matrix parsing should
        succeed",
        );

    assert_eq!(
        golden_distance_data.len(),
        parsed_square_matrix.raw_distances().len()
    );

    for (i, &distance) in golden_distance_data.iter().enumerate() {
        assert_eq!(
            distance,
            parsed_square_matrix.raw_distances()[i],
            "Distance data mismatch at index {} with values {:?} (expected) vs {:?} (actual)",
            i,
            distance,
            parsed_square_matrix.raw_distances()[i]
        );
    }

    assert_eq!(parsed_square_matrix.raw_distances(), golden_distance_data);

    for row in 0..parsed_square_matrix.distance_matrix().dimension() {
        for col in 0..=row {
            assert_eq!(
                parsed_square_matrix
                    .distance_matrix()
                    .get_data(Node(row), Node(col)),
                parsed_triangular_matrix
                    .distance_matrix()
                    .get_data(Node(row), Node(col)),
                "Distance matrix mismatch at position ({}, {}) with values {:?} (symmetric) vs \
                 {:?} (matrix)",
                row,
                col,
                parsed_triangular_matrix
                    .distance_matrix()
                    .get_data(Node(row), Node(col)),
                parsed_square_matrix
                    .distance_matrix()
                    .get_data(Node(row), Node(col))
            );
        }
    }
}

fn parse_golden_file_square_matrix(instance_path: &str) -> Option<Vec<Distance>> {
    let golden_file_path = format!(
        "tests/test_assets/distances_square_matrix/{}.txt",
        instance_path
            .split('/')
            .last()
            .unwrap()
            .strip_suffix(".tsp")
            .unwrap()
    );
    let Ok(file) = File::open(&golden_file_path) else {
        return None;
    };
    Some(
        BufReader::new(file)
            .lines()
            .map(|line| {
                let line = line.unwrap();
                line.split(" ")
                    .filter(|entry| !entry.is_empty())
                    .map(|entry| Distance(entry.trim().parse::<i32>().unwrap()))
                    .collect::<Vec<Distance>>()
                    .into_iter()
            })
            .flatten()
            .collect::<Vec<Distance>>(),
    )
}

// This doesn't actually need to be run on all instances, but might as well reuse the macro since
// we have it.
test_fn_on_all_instances!(
    check_input_file_against_golden_file,
    check_matrix_parsing,
    0,
    550
);
