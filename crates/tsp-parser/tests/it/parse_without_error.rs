use std::any::Any;

use tsp_core::instance::{
    distance::Distance,
    matrix::{Matrix, MatrixSym},
};
use tsp_macros::test_fn_on_instances_filtered;

fn parse_instance_symmetric(path: &str) {
    let parsing_result = std::panic::catch_unwind(|| {
        tsp_parser::parse_tsp_instance::<MatrixSym<Distance>>(path.to_owned())
    });
    if let Err(err) = parsing_result {
        handle_error(err);
    } else {
        assert!(parsing_result.is_ok());
    }
}

fn parse_instance_non_symmetric(path: &str) {
    let parsing_result = std::panic::catch_unwind(|| {
        tsp_parser::parse_tsp_instance::<Matrix<Distance>>(path.to_owned())
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
        _ => assert!(false, "Parsing failed with unexpected error: {}", err_msg),
    }
}

test_fn_on_instances_filtered!(parse_instance_symmetric, short_symmetric, 0, 50);
test_fn_on_instances_filtered!(parse_instance_non_symmetric, short_non_symmetric, 0, 50);
test_fn_on_instances_filtered!(parse_instance_symmetric, symmetric, 51, 10000);
test_fn_on_instances_filtered!(parse_instance_non_symmetric, non_symmetric, 51, 10000);
