use concorde_rs::solver::tsp_hk;
use criterion::{Criterion, criterion_group, criterion_main};
use tsp_parser::parse_tsp_instance;
use tsp_solvers::held_karp::held_karp;

fn held_karp_benchmark_og(c: &mut Criterion) {
    let tsp_instance = parse_tsp_instance("../../instances/bench/12.tsp").unwrap();
    let lower_distance_matrix = concorde_rs::LowerDistanceMatrix {
        num_nodes: tsp_instance.metadata().dimension as u32,
        values: tsp_instance
            .raw_distances()
            .iter()
            .map(|&d| d.0 as u32)
            .collect(),
    };

    c.bench_function("Held Karp using concorde_rs", |b| {
        b.iter(|| tsp_hk(&lower_distance_matrix).unwrap())
    });
}

fn held_karp_benchmark(c: &mut Criterion) {
    let tsp_instance = parse_tsp_instance("../../instances/bench/12.tsp").unwrap();
    let non_symmetric_matrix = tsp_instance.distances().to_non_symmetric();

    c.bench_function("Held Karp using own implementation", |b| {
        b.iter(|| held_karp(&non_symmetric_matrix).unwrap())
    });
}

criterion_group!(held_karp_bench, held_karp_benchmark_og, held_karp_benchmark);
criterion_main!(held_karp_bench);
