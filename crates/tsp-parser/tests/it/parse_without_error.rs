use std::any::Any;

use tsp_core::instance::{
    distance::Distance,
    matrix::{SquareMatrix, TriangularMatrix},
};
use tsp_macros::test_fn_on_all_instances;

fn parse_instance_triangular(path: &str) {
    let parsing_result = std::panic::catch_unwind(|| {
        tsp_parser::parse_tsp_instance::<TriangularMatrix<Distance>>(path.to_owned())
    });
    if let Err(err) = parsing_result {
        handle_error(err);
    } else {
        assert!(parsing_result.is_ok());
    }
}

fn parse_instance_square(path: &str) {
    let parsing_result = std::panic::catch_unwind(|| {
        tsp_parser::parse_tsp_instance::<SquareMatrix<Distance>>(path.to_owned())
    });
    if let Err(err) = parsing_result {
        handle_error(err);
    } else {
        assert!(parsing_result.is_ok());
    }
}

fn handle_error(err: Box<dyn Any + Send>) {
    let err_msg = if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic message".to_string()
    };

    match err_msg.as_str() {
        "not yet implemented: Explicit distance matrix parsing is not supported yet" => {}
        "not yet implemented: Fixed edges sections are not supported yet" => {}
        "not yet implemented: Parsing explicit full matrix into square matrix is not \
         implemented yet." => {}
        _ => {
            if !err_msg.starts_with("not yet implemented: Explicit edge weight format") {
                panic!("Parsing failed with unexpected error: {}", err_msg)
            }
        }
    }
}

test_fn_on_all_instances!(parse_instance_triangular, short_triangular, 0, 40);
test_fn_on_all_instances!(parse_instance_square, short_square, 0, 40);
test_fn_on_all_instances!(parse_instance_triangular, triangular, 41, 10000);
test_fn_on_all_instances!(parse_instance_square, square, 41, 10000);
