use criterion::{Criterion, criterion_group, criterion_main};
use tsp_core::instance::{distance::Distance, matrix::MatrixSym};
use tsp_parser::parse_tsp_instance;

fn a280_benchmark(c: &mut Criterion) {
    c.bench_function("Parse and compute distances \"a280.tsp\"", |b| {
        b.iter(|| {
            parse_tsp_instance::<MatrixSym<Distance>>("../../instances/tsplib_symmetric/a280.tsp")
                .unwrap()
        })
    });
}

fn d198_benchmark(c: &mut Criterion) {
    c.bench_function("Parse and compute distances \"d198.tsp\"", |b| {
        b.iter(|| {
            parse_tsp_instance::<MatrixSym<Distance>>("../../instances/tsplib_symmetric/d198.tsp")
                .unwrap()
        })
    });
}

fn d493_benchmark(c: &mut Criterion) {
    c.bench_function("Parse and compute distances \"d493.tsp\"", |b| {
        b.iter(|| {
            parse_tsp_instance::<MatrixSym<Distance>>("../../instances/tsplib_symmetric/d493.tsp")
                .unwrap()
        })
    });
}

fn d1291_benchmark(c: &mut Criterion) {
    c.bench_function("Parse and compute distances \"d1291.tsp\"", |b| {
        b.iter(|| {
            parse_tsp_instance::<MatrixSym<Distance>>("../../instances/tsplib_symmetric/d1291.tsp")
                .unwrap()
        })
    });
}

fn d18512_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("d18512_parsing");
    group.sample_size(20);
    group.bench_function("Parse and compute distances \"d18512.tsp\"", |b| {
        b.iter(|| {
            parse_tsp_instance::<MatrixSym<Distance>>("../../instances/tsplib_symmetric/d18512.tsp")
                .unwrap()
        })
    });
    group.finish();
}

criterion_group!(a280, a280_benchmark);
criterion_group!(d198, d198_benchmark);
criterion_group!(d493, d493_benchmark);
criterion_group!(d1291, d1291_benchmark);
criterion_group!(d18512, d18512_benchmark);
criterion_main!(d198, a280, d493, d1291, d18512);
