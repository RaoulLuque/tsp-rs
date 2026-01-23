use criterion::{Criterion, criterion_group, criterion_main};
use tsp_core::instance::{distance::Distance, matrix::TriangularMatrix};
use tsp_parser::parse_tsp_instance;
use tsp_solvers::held_karp_mod::held_karp_parallel;

fn att48_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("held_karp_parallel");
    group.sample_size(10);

    let tsp_instance =
        parse_tsp_instance::<TriangularMatrix<Distance>>("../../instances/tsplib_symmetric/att48.tsp")
            .unwrap();
    let non_symmetric_matrix = tsp_instance.distance_matrix().to_edge_data_matrix();

    group.bench_function("Held Karp Parallel: att48.tsp", |b| {
        b.iter(|| held_karp_parallel(&non_symmetric_matrix).unwrap())
    });
    group.finish();
}

fn berlin52_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("held_karp_parallel");
    group.sample_size(10);

    let tsp_instance =
        parse_tsp_instance::<TriangularMatrix<Distance>>("../../instances/tsplib_symmetric/berlin52.tsp")
            .unwrap();
    let non_symmetric_matrix = tsp_instance.distance_matrix().to_edge_data_matrix();

    group.bench_function("Held Karp Parallel: berlin52.tsp", |b| {
        b.iter(|| held_karp_parallel(&non_symmetric_matrix).unwrap())
    });
    group.finish();
}

criterion_group!(
    held_karp_parallel_benchmarks,
    att48_parallel,
    berlin52_parallel
);
criterion_main!(held_karp_parallel_benchmarks);
