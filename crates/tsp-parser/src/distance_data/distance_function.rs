use tsp_core::{instance::distance::Distance, tsp_lib_spec::EdgeWeightType};

pub fn get_distance_function(
    edge_weight_type: &EdgeWeightType,
) -> impl Fn(&(f64, f64), &(f64, f64)) -> Distance + Send + Sync + Copy {
    use EdgeWeightType::*;
    match edge_weight_type {
        EUC_2D => compute_euclidean_distance,
        _ => unimplemented!(
            "Distance function for edge weight type {:?} is not yet implemented",
            edge_weight_type
        ),
    }
}

/// Computes the Euclidean distance between two points as defined in TSPLIB95.
#[inline(always)]
pub fn compute_euclidean_distance(point_a: &(f64, f64), point_b: &(f64, f64)) -> Distance {
    Distance(nint(
        ((point_a.0 - point_b.0).powi(2) + (point_a.1 - point_b.1).powi(2)).sqrt(),
    ))
}

/// Nearest integer function as defined in TSPLIB95.
///
/// Expects a non-negative float input.
#[inline(always)]
pub fn nint(x: f64) -> i32 {
    (x + 0.5) as i32
}
