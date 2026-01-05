use criterion::{Criterion, criterion_group, criterion_main};
use tsp_core::instance::{
    distance::Distance,
    matrix::{Matrix, MatrixSym},
};
use tsp_parser::parse_tsp_instance;

macro_rules! create_held_karp_benchmarks {
    ($file_path:expr, $name_sym:ident, $name_non_sym:ident, $name_group:ident) => {
        fn $name_sym(c: &mut Criterion) {
            c.bench_function(
                concat!("Parse \"", $file_path, ".tsp\" into symmetric"),
                |b| {
                    b.iter(|| {
                        parse_tsp_instance::<MatrixSym<Distance>>(concat!(
                            "../../instances/",
                            $file_path,
                        ))
                        .unwrap()
                    })
                },
            );
        }

        fn $name_non_sym(c: &mut Criterion) {
            c.bench_function(
                concat!("Parse \"", $file_path, ".tsp\" into non-symmetric"),
                |b| {
                    b.iter(|| {
                        parse_tsp_instance::<Matrix<Distance>>(concat!(
                            "../../instances/",
                            $file_path,
                        ))
                        .unwrap()
                    })
                },
            );
        }

        criterion_group!($name_group, $name_sym, $name_non_sym);
    };
}

create_held_karp_benchmarks!(
    "tsplib_symmetric/a280.tsp",
    parse_a280_into_symmetric,
    parse_a280_into_non_symmetric,
    a280
);
create_held_karp_benchmarks!(
    "tsplib_symmetric/d198.tsp",
    parse_d198_into_symmetric,
    parse_d198_into_non_symmetric,
    d198
);
create_held_karp_benchmarks!(
    "tsplib_symmetric/d493.tsp",
    parse_d493_into_symmetric,
    parse_d493_into_non_symmetric,
    d493
);
create_held_karp_benchmarks!(
    "tsplib_symmetric/d1291.tsp",
    parse_d1291_into_symmetric,
    parse_d1291_into_non_symmetric,
    d1291
);

fn parse_d18512_into_symmetric(c: &mut Criterion) {
    let mut group = c.benchmark_group("d18512_parsing");
    group.sample_size(20);
    group.bench_function("Parse \"d18512.tsp\" into symmetric", |b| {
        b.iter(|| {
            parse_tsp_instance::<MatrixSym<Distance>>("../../instances/tsplib_symmetric/d18512.tsp")
                .unwrap()
        })
    });
    group.finish();
}

fn parse_d18512_into_non_symmetric(c: &mut Criterion) {
    let mut group = c.benchmark_group("d18512_parsing");
    group.sample_size(20);
    group.bench_function("Parse \"d18512.tsp\" into non-symmetric", |b| {
        b.iter(|| {
            parse_tsp_instance::<Matrix<Distance>>("../../instances/tsplib_symmetric/d18512.tsp")
                .unwrap()
        })
    });
    group.finish();
}

criterion_group!(
    d18512,
    parse_d18512_into_symmetric,
    parse_d18512_into_non_symmetric
);
criterion_main!(a280, d198, d493, d1291, d18512);
