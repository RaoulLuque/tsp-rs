use criterion::{BatchSize::SmallInput, Criterion, criterion_group, criterion_main};
use tsp_core::instance::{distance::ScaledDistance, matrix::Matrix};
use tsp_parser::parse_tsp_instance;
use tsp_solvers::held_karp_mod::{EdgeState, min_one_tree as min_one_tree_function};

fn min_one_tree_benchmark(c: &mut Criterion) {
    let tsp_instance = parse_tsp_instance("../../instances/tsplib_symmetric/a280.tsp").unwrap();
    let distances_non_symmetric = tsp_instance.distance_matrix().to_edge_data_matrix();
    let scaled_distances = Matrix::new(
        distances_non_symmetric
            .data()
            .iter()
            .map(|&d| ScaledDistance::from_distance(d))
            .collect::<Vec<_>>(),
        distances_non_symmetric.dimension(),
    );
    let edge_states =
        Matrix::new_from_dimension_with_value(scaled_distances.dimension(), EdgeState::Available);
    let node_penalties = vec![ScaledDistance(0); scaled_distances.dimension()];

    c.bench_function("Compute min one tree", |b| {
        b.iter_batched_ref(
            || node_penalties.clone(),
            |node_penalties| min_one_tree_function(&scaled_distances, &edge_states, node_penalties),
            SmallInput,
        )
    });
}

criterion_group!(min_one_tree, min_one_tree_benchmark);
criterion_main!(min_one_tree);
