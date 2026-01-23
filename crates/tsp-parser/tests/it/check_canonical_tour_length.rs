use tsp_core::instance::{
    distance::Distance,
    matrix::{SquareMatrix, TriangularMatrix},
};
use tsp_macros::test_fn_on_all_instances;

fn check_against_canonical_tour_length(instance_path: &str) {
    let golden_file_path = format!(
        "tests/test_assets/tour_lengths/{}.txt",
        instance_path
            .split('/')
            .last()
            .unwrap()
            .strip_suffix(".tsp")
            .unwrap()
    );

    if let Ok(golden_length_str) = std::fs::read_to_string(&golden_file_path) {
        let golden_length = Distance(
            golden_length_str
                .trim()
                .parse::<i32>()
                .expect("Golden file should contain a valid distance"),
        );

        let tsp_instance_sym =
            tsp_parser::parse_tsp_instance::<TriangularMatrix<Distance>>(instance_path)
                .expect("Failed to parse TSP instance");
        let tsp_instance_matrix =
            tsp_parser::parse_tsp_instance::<SquareMatrix<Distance>>(instance_path)
                .expect("Failed to parse TSP instance");

        let mut len_sym = Distance(0);
        let mut len_matrix = Distance(0);

        let dimension = tsp_instance_sym.metadata().dimension;

        for i in 0..dimension {
            let from = i;
            let to = (i + 1) % dimension;
            len_sym = len_sym
                + tsp_instance_sym
                    .distance_matrix()
                    .get_data(from.into(), to.into());
            len_matrix = len_matrix
                + tsp_instance_matrix
                    .distance_matrix()
                    .get_data(from.into(), to.into());
        }
        assert_eq!(
            len_sym, golden_length,
            "Symmetric matrix length does not match golden length"
        );
        assert_eq!(
            len_matrix, golden_length,
            "Matrix length does not match golden length"
        );
    }
}

// This doesn't actually need to be run on all instances, but might as well reuse the macro since
// we have it.
test_fn_on_all_instances!(
    check_against_canonical_tour_length,
    check_canonical_tour_length,
    0,
    550
);
