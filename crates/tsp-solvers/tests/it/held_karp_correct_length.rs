use tsp_core::instance::{TSPSymInstance, distance::Distance, matrix::SquareMatrix};
use tsp_solvers::held_karp;

fn check_correct_length_for_held_karp(instance_path: &str) {
    let tsp_instance: TSPSymInstance<SquareMatrix<Distance>> =
        tsp_parser::parse_tsp_instance(instance_path).unwrap();
    let best_tour = held_karp(&tsp_instance.distance_matrix()).unwrap();
    let length = find_length_in_golden_file(instance_path);

    assert_eq!(
        best_tour.cost.0, length,
        "Held-Karp computed tour length {} does not match canonical tour length {} for instance {}",
        best_tour.cost.0, length, instance_path
    );
}

fn find_length_in_golden_file(instance_path: &str) -> i32 {
    let golden_file_path = format!("tests/test_assets/tour_lengths/solutions.txt",);
    let file_name = instance_path
        .split('/')
        .last()
        .unwrap()
        .strip_suffix(".tsp")
        .unwrap();
    for line in std::fs::read_to_string(&golden_file_path)
        .expect("Failed to read golden file")
        .lines()
    {
        let mut parts = line.split(':');
        let name = parts.next().unwrap().trim();
        let length_str = parts.next().unwrap().trim();
        if name == file_name {
            return length_str
                .parse::<i32>()
                .expect("Golden file should contain a valid distance");
        }
    }

    panic!(
        "Instance {} not found in golden file {}",
        instance_path, golden_file_path
    );
}

// // This doesn't actually need to be run on all instances, but might as well reuse the macro since
// // we have it.
// tsp_macros::test_fn_on_all_instances!(
//     check_correct_length_for_held_karp,
//     held_karp_correct_length,
//     0,
//     60
// );
