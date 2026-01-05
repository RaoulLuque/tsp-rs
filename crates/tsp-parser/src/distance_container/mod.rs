use tsp_core::instance::{
    InstanceMetadata,
    distance::Distance,
    matrix::{Matrix, MatrixSym},
};

mod matrix_sym;

pub trait ParseFromTSPLib {
    fn from_node_coord_section<PointType: Sync + Send>(
        node_data: &Vec<PointType>,
        metadata: &InstanceMetadata,
        distance_function: impl Fn(&PointType, &PointType) -> Distance + Sync + Send + Copy,
    ) -> Self;
}

impl ParseFromTSPLib for Matrix<Distance> {
    fn from_node_coord_section<PointType: Sync + Send>(
        node_data: &Vec<PointType>,
        metadata: &InstanceMetadata,
        distance_function: impl Fn(&PointType, &PointType) -> Distance + Sync + Send + Copy,
    ) -> Self {
        let sym_matrix =
            MatrixSym::<Distance>::from_node_coord_section(node_data, metadata, distance_function);

        sym_matrix.to_edge_data_matrix()
    }
}
