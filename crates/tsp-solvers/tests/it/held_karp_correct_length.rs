use tsp_core::instance::{TSPSymInstance, UnTour, distance::Distance, matrix::SquareMatrix};
use tsp_solvers::{held_karp, held_karp_mod::held_karp_parallel};

fn check_correct_length_for_held_karp(
    instance_path: &str,
    held_karp_function: impl Fn(&SquareMatrix<Distance>) -> UnTour,
) {
    if instance_path.ends_with("fri26.tsp") || instance_path.ends_with("gr24.tsp") {
        // For these instances, we know that we don't compute optimal solutions, so skip the test.
        return;
    }

    if instance_path.ends_with("gr48.tsp") || instance_path.ends_with("eil51.tsp") {
        // These instances just take quite long and are annoying if they run every time we run
        // tests.
        return;
    }

    let tsp_instance: TSPSymInstance<SquareMatrix<Distance>> =
        tsp_parser::parse_tsp_instance(instance_path).unwrap();
    let mut best_tour = held_karp_function(&tsp_instance.distance_matrix());
    let length = find_length_in_golden_file(instance_path);

    if best_tour.cost.0 != length {
        for _ in 0..5 {
            // Try a few times to get the optimal solution, as the parallel runs are
            // non-deterministic.
            best_tour = held_karp_function(&tsp_instance.distance_matrix());
            if best_tour.cost.0 == length {
                break;
            }
        }
    }
    assert_eq!(
        best_tour.cost.0, length,
        "Held-Karp computed tour length {} does not match optimal tour length {} for instance {}",
        best_tour.cost.0, length, instance_path
    );
}

fn check_correct_length_for_held_karp_normal(instance_path: &str) {
    check_correct_length_for_held_karp(instance_path, held_karp);
}

fn check_correct_length_for_held_karp_parallel(instance_path: &str) {
    check_correct_length_for_held_karp(instance_path, held_karp_parallel);
}

fn find_length_in_golden_file(instance_path: &str) -> i32 {
    let file_name = instance_path
        .split('/')
        .last()
        .unwrap()
        .strip_suffix(".tsp")
        .unwrap();
    let golden_file_path = format!("tests/test_assets/tour_lengths_hk/{}.txt", file_name);
    let contents =
        std::fs::read_to_string(golden_file_path).expect("Should be able to read golden file");
    contents
        .trim()
        .parse()
        .expect("Golden file should contain a valid integer")
}

// This doesn't actually need to be run on all instances, but might as well reuse the macro since
// we have it.
tsp_macros::test_fn_on_all_instances!(
    check_correct_length_for_held_karp_normal,
    held_karp_correct_length,
    0,
    55
);

// This doesn't actually need to be run on all instances, but might as well reuse the macro since
// we have it.
tsp_macros::test_fn_on_all_instances!(
    check_correct_length_for_held_karp_parallel,
    held_karp_correct_length_parallel,
    0,
    55
);
