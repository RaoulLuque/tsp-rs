use tsp_core::instance::{InstanceMetadata, distance::Distance};

use crate::data_section::{GeoPoint, Point2D};

mod matrix_sym;

pub trait ParseFromTSPLib {
    fn from_2d_node_coord_section(
        node_data: &Vec<Point2D>,
        metadata: &InstanceMetadata,
        distance_function: impl Fn(&Point2D, &Point2D) -> Distance + Sync + Send + Copy,
    ) -> Self;

    fn from_geo_node_coord_section(
        node_data: &Vec<GeoPoint>,
        metadata: &InstanceMetadata,
        distance_function: impl Fn(&GeoPoint, &GeoPoint) -> Distance + Sync + Send + Copy,
    ) -> Self;
}
