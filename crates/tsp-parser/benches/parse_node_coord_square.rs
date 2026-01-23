use criterion::{Criterion, criterion_group, criterion_main};
use tsp_core::instance::{distance::Distance, matrix::SquareMatrix};
use tsp_parser::parse_tsp_instance;

macro_rules! create_parse_benchmark_square {
    ($file_path:expr, $name_square:ident, $name_group:ident) => {
        fn $name_square(c: &mut Criterion) {
            c.bench_function(concat!("Parse \"", $file_path, "\" into square"), |b| {
                b.iter(|| {
                    parse_tsp_instance::<SquareMatrix<Distance>>(concat!(
                        "../../instances/",
                        $file_path,
                    ))
                    .unwrap()
                })
            });
        }

        criterion_group!($name_group, $name_square);
    };
}

create_parse_benchmark_square!("tsplib_symmetric/a280.tsp", parse_a280_into_square, a280);
create_parse_benchmark_square!("tsplib_symmetric/d198.tsp", parse_d198_into_square, d198);
create_parse_benchmark_square!("tsplib_symmetric/d493.tsp", parse_d493_into_square, d493);

criterion_main!(a280, d198, d493);
