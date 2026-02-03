use criterion::{Criterion, criterion_group, criterion_main};
use tsp_core::instance::{distance::Distance, matrix::SquareMatrix};
use tsp_parser::parse_tsp_instance;
use tsp_solvers::held_karp_mod::held_karp_parallel;

macro_rules! create_held_karp_parallel_benchmarks {
    ($file_path:expr, $name_hk_parallel:ident) => {
        fn $name_hk_parallel(c: &mut Criterion) {
            let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("instances")
                .join($file_path);

            let tsp_instance =
                parse_tsp_instance::<SquareMatrix<Distance>>(path.to_str().unwrap()).unwrap();

            c.bench_function(
                concat!("Held Karp Parallel: ", $file_path),
                |b| b.iter(|| held_karp_parallel(&tsp_instance.distance_matrix())),
            );
        }
    };
}

create_held_karp_parallel_benchmarks!("tsp_rust/12.tsp", held_karp_own_12);
create_held_karp_parallel_benchmarks!("tsplib_symmetric/gr17.tsp", held_karp_own_gr17);
create_held_karp_parallel_benchmarks!("tsplib_symmetric/bays29.tsp", held_karp_own_bays29);
create_held_karp_parallel_benchmarks!("tsplib_symmetric/berlin52.tsp", held_karp_own_berlin52);

criterion_group!(held_karp_bench_12, held_karp_own_12);
criterion_group!(held_karp_bench_gr17, held_karp_own_gr17);
criterion_group!(held_karp_bench_bays29, held_karp_own_bays29);
criterion_group!(held_karp_bench_berlin52, held_karp_own_berlin52);

criterion_main!(
    held_karp_bench_12,
    held_karp_bench_gr17,
    held_karp_bench_bays29,
    held_karp_bench_berlin52
);
