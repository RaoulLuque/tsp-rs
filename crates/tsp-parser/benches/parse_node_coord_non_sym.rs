use criterion::{Criterion, criterion_group, criterion_main};
use tsp_core::instance::{distance::Distance, matrix::SquareMatrix};
use tsp_parser::parse_tsp_instance;

macro_rules! create_parse_benchmark_non_symmetric {
    ($file_path:expr, $name_non_sym:ident, $name_group:ident) => {
        fn $name_non_sym(c: &mut Criterion) {
            c.bench_function(
                concat!("Parse \"", $file_path, "\" into non-symmetric"),
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

        criterion_group!($name_group, $name_non_sym);
    };
}

create_parse_benchmark_non_symmetric!(
    "tsplib_symmetric/a280.tsp",
    parse_a280_into_non_symmetric,
    a280
);
create_parse_benchmark_non_symmetric!(
    "tsplib_symmetric/d198.tsp",
    parse_d198_into_non_symmetric,
    d198
);
create_parse_benchmark_non_symmetric!(
    "tsplib_symmetric/d493.tsp",
    parse_d493_into_non_symmetric,
    d493
);

criterion_main!(a280, d198, d493);
