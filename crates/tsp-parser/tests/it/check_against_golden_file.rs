use std::{
    self,
    fs::File,
    io::{BufRead, BufReader},
};

use tsp_core::instance::{
    TSPSymInstance,
    distance::Distance,
    matrix::{Matrix, MatrixSym},
    node::Node,
};

fn check_input_file_against_golden_file(file_name: &str) {
    let input_instance_sym: TSPSymInstance<MatrixSym<Distance>> =
        tsp_parser::parse_tsp_instance("../../instances/".to_owned() + file_name + ".tsp")
            .expect("Symmetric parsing should succeed");
    let input_instance_matrix: TSPSymInstance<Matrix<Distance>> =
        tsp_parser::parse_tsp_instance("../../instances/".to_owned() + file_name + ".tsp")
            .expect("Matrix parsing should succeed");
    let golden_distance_data = BufReader::new(
        File::open(
            "tests/test_assets/distances/".to_owned()
                + file_name.split("/").last().unwrap()
                + ".txt",
        )
        .unwrap(),
    )
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
    println!(
        "Symmetric Matrix: \n{}",
        input_instance_sym.distance_matrix()
    );
    println!("Matrix: \n{}", input_instance_matrix.distance_matrix());
    assert_eq!(input_instance_sym.raw_distances(), golden_distance_data);
    for row in 0..input_instance_matrix.distance_matrix().dimension() {
        for col in 0..input_instance_matrix.distance_matrix().dimension() {
            assert_eq!(
                input_instance_matrix
                    .distance_matrix()
                    .get_data(Node(row), Node(col)),
                input_instance_sym
                    .distance_matrix()
                    .get_data(Node(row), Node(col)),
                "Distance matrix mismatch at position ({}, {}) with values {:?} (symmetric) vs \
                 {:?} (matrix)",
                row,
                col,
                input_instance_sym
                    .distance_matrix()
                    .get_data(Node(row), Node(col)),
                input_instance_matrix
                    .distance_matrix()
                    .get_data(Node(row), Node(col))
            );
        }
    }
}

#[test]
fn test_12_short() {
    check_input_file_against_golden_file("tsp_rust/12");
}

#[test]
fn test_ulysses22_short() {
    check_input_file_against_golden_file("tsplib_symmetric/ulysses22");
}

#[test]
fn test_berlin52() {
    check_input_file_against_golden_file("tsplib_symmetric/berlin52");
}

#[test]
fn test_a280() {
    check_input_file_against_golden_file("tsplib_symmetric/a280");
}

#[test]
fn test_d198() {
    check_input_file_against_golden_file("tsplib_symmetric/d198");
}

#[test]
fn test_d493() {
    check_input_file_against_golden_file("tsplib_symmetric/d493");
}
