use tsp_core::instance::{InstanceMetadata, distance::Distance};

mod matrix_sym;

pub trait ParseFromTSPLib {
    fn from_node_coord_section(
        node_data: &Vec<(f64, f64)>,
        metadata: &InstanceMetadata,
        distance_function: impl Fn(&(f64, f64), &(f64, f64)) -> Distance + Sync + Send + Copy,
    ) -> Self;
}
