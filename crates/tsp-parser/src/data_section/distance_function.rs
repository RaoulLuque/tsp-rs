use std::f64::consts::PI;

use tsp_core::{instance::distance::Distance, tsp_lib_spec::EdgeWeightType};

use crate::data_section::{GeoPoint, Point2D, Point3D};

pub fn get_2d_distance_function(
    edge_weight_type: &EdgeWeightType,
) -> impl Fn(&Point2D, &Point2D) -> Distance + Send + Sync + Copy {
    use EdgeWeightType::*;
    match edge_weight_type {
        EUC_2D => euclidean_distance_2d,
        MAX_2D => max_distance_2d,
        MAN_2D => manhattan_distance_2d,
        CEIL_2D => ceil_distance_2d,
        ATT => att_distance_2d,
        _ => unreachable!(
            "Distance function for edge weight type {:?} is not 2D.",
            edge_weight_type
        ),
    }
}

/// Computes the 2D Euclidean distance between two points as defined in TSPLIB95.
#[inline(always)]
pub fn euclidean_distance_2d(point_a: &Point2D, point_b: &Point2D) -> Distance {
    Distance(nint(
        ((point_a.x - point_b.x).powi(2) + (point_a.y - point_b.y).powi(2)).sqrt(),
    ))
}

/// Computes the 3D Euclidean distance between two points as defined in TSPLIB95.
#[inline(always)]
pub fn euclidean_distance_3d(point_a: &Point3D, point_b: &Point3D) -> Distance {
    Distance(nint(
        ((point_a.x - point_b.x).powi(2)
            + (point_a.y - point_b.y).powi(2)
            + (point_a.z - point_b.z).powi(2))
        .sqrt(),
    ))
}

/// Computes the 2D Max distance between two points as defined in TSPLIB95.
#[inline(always)]
pub fn max_distance_2d(point_a: &Point2D, point_b: &Point2D) -> Distance {
    Distance(nint(
        (point_a.x - point_b.x)
            .abs()
            .max((point_a.y - point_b.y).abs()),
    ))
}

/// Computes the 3D Max distance between two points as defined in TSPLIB95.
#[inline(always)]
pub fn max_distance_3d(point_a: &Point3D, point_b: &Point3D) -> Distance {
    Distance(nint(
        (point_a.x - point_b.x)
            .abs()
            .max((point_a.y - point_b.y).abs())
            .max((point_a.z - point_b.z).abs()),
    ))
}

/// Computes the 2D Manhattan distance between two points as defined in TSPLIB95.
#[inline(always)]
pub fn manhattan_distance_2d(point_a: &Point2D, point_b: &Point2D) -> Distance {
    Distance(nint(
        (point_a.x - point_b.x).abs() + (point_a.y - point_b.y).abs(),
    ))
}

/// Computes the 3D Manhattan distance between two points as defined in TSPLIB95.
#[inline(always)]
pub fn manhattan_distance_3d(point_a: &Point3D, point_b: &Point3D) -> Distance {
    Distance(nint(
        (point_a.x - point_b.x).abs()
            + (point_a.y - point_b.y).abs()
            + (point_a.z - point_b.z).abs(),
    ))
}

/// Computes the 2D Ceil distance between two points as defined in TSPLIB95.
#[inline(always)]
pub fn ceil_distance_2d(point_a: &Point2D, point_b: &Point2D) -> Distance {
    Distance(
        ((point_a.x - point_b.x).powi(2) + (point_a.y - point_b.y).powi(2))
            .sqrt()
            .ceil() as i32,
    )
}

/// Computes the geographical distance between two points as defined in TSPLIB95.
#[inline(always)]
pub fn geographical_distance(point_a: &GeoPoint, point_b: &GeoPoint) -> Distance {
    let rrr = 6378.388;
    let q1 = (point_a.longitude - point_b.longitude).cos();
    let q2 = (point_a.latitude - point_b.latitude).cos();
    let q3 = (point_a.latitude + point_b.latitude).cos();

    Distance((rrr * ((0.5 * ((1.0 + q1) * q2 - (1.0 - q1) * q3)).acos() + 1.0)) as i32)
}

/// Converts a 2D point representing geographical coordinates as defined in TSPLIB95.
pub fn convert_to_geo_coordinates(point: &Point2D) -> GeoPoint {
    let deg_lat = nint(point.x) as f64;
    let min_lat = point.x - deg_lat;
    let latitude = PI * (deg_lat + (5.0 * min_lat) / 3.0) / 180.0;

    let deg_lon = point.y.floor();
    let min_lon = point.y - deg_lon;
    let longitude = std::f64::consts::PI * (deg_lon + (5.0 * min_lon) / 3.0) / 180.0;

    GeoPoint {
        latitude,
        longitude,
    }
}

/// Computes the ATT distance between two points as defined in TSPLIB95.
#[inline(always)]
pub fn att_distance_2d(point_a: &Point2D, point_b: &Point2D) -> Distance {
    let rij = ((point_a.x - point_b.x).powi(2) + (point_a.y - point_b.y).powi(2)).sqrt() / 10.0;
    let tij = nint(rij);
    if (tij as f64) < rij {
        Distance(tij + 1)
    } else {
        Distance(tij)
    }
}

/// Nearest integer function as defined in TSPLIB95.
///
/// Expects a non-negative float input.
#[inline(always)]
pub fn nint(x: f64) -> i32 {
    (x + 0.5) as i32
}
