use concorde_rs::solver::tsp_hk;
use criterion::{Criterion, criterion_group, criterion_main};
use tsp_core::instance::{distance::Distance, matrix::TriangularMatrix};
use tsp_parser::parse_tsp_instance;
use tsp_solvers::held_karp;

macro_rules! create_held_karp_benchmarks {
    ($file_path:expr, $name_concorde:ident, $name_own:ident) => {
        fn $name_concorde(c: &mut Criterion) {
            let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("instances")
                .join($file_path);

            let tsp_instance =
                parse_tsp_instance::<TriangularMatrix<Distance>>(path.to_str().unwrap()).unwrap();
            let lower_distance_matrix = concorde_rs::LowerDistanceMatrix {
                num_nodes: tsp_instance.metadata().dimension as u32,
                values: tsp_instance
                    .raw_distances()
                    .iter()
                    .map(|&d| d.0 as u32)
                    .collect(),
            };

            c.bench_function(concat!("Held Karp using concorde_rs: ", $file_path), |b| {
                b.iter(|| tsp_hk(&lower_distance_matrix).unwrap())
            });
        }

        fn $name_own(c: &mut Criterion) {
            let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("instances")
                .join($file_path);

            let tsp_instance =
                parse_tsp_instance::<TriangularMatrix<Distance>>(path.to_str().unwrap()).unwrap();
            let non_symmetric_matrix = tsp_instance.distance_matrix().to_edge_data_matrix();

            c.bench_function(
                concat!("Held Karp using own implementation: ", $file_path),
                |b| b.iter(|| held_karp(&non_symmetric_matrix).unwrap()),
            );
        }
    };
}

create_held_karp_benchmarks!("tsp_rust/12.tsp", held_karp_concorde_12, held_karp_own_12);
create_held_karp_benchmarks!(
    "tsplib_symmetric/gr17.tsp",
    held_karp_concorde_gr17,
    held_karp_own_gr17
);
create_held_karp_benchmarks!(
    "tsplib_symmetric/bays29.tsp",
    held_karp_concorde_bays29,
    held_karp_own_bays29
);

criterion_group!(held_karp_bench_12, held_karp_concorde_12, held_karp_own_12);
criterion_group!(
    held_karp_bench_gr17,
    held_karp_concorde_gr17,
    held_karp_own_gr17
);
criterion_group!(
    held_karp_bench_bays29,
    held_karp_concorde_bays29,
    held_karp_own_bays29
);

criterion_main!(held_karp_bench_12, held_karp_bench_gr17, held_karp_bench_bays29);
