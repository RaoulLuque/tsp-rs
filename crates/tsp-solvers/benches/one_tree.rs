use criterion::{BatchSize::SmallInput, Criterion, criterion_group, criterion_main};
use tsp_core::instance::edge::{data::EdgeDataMatrix, distance::ScaledDistance};
use tsp_parser::parse_tsp_instance;
use tsp_solvers::held_karp::{EdgeState, trees::min_one_tree as min_one_tree_function};

fn min_one_tree_benchmark(c: &mut Criterion) {
    let tsp_instance = parse_tsp_instance("../../instances/bench/a280.tsp").unwrap();
    let distances_non_symmetric = tsp_instance.distances().to_non_symmetric();
    let scaled_distances = EdgeDataMatrix {
        dimension: distances_non_symmetric.dimension,
        data: distances_non_symmetric
            .data
            .iter()
            .map(|&d| ScaledDistance::from_distance(d))
            .collect::<Vec<_>>(),
    };
    let edge_states = EdgeDataMatrix {
        data: vec![EdgeState::Available; scaled_distances.data.len()],
        dimension: scaled_distances.dimension,
    };
    let node_penalties = vec![ScaledDistance(0); scaled_distances.dimension];

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
