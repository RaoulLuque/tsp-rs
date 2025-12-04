pub struct Symmetric;
pub struct Asymmetric;

mod directionality_seal {
    pub trait Directionality {}
}
// Re-export the trait for this crate
pub(crate) use directionality_seal::Directionality;

impl Directionality for Symmetric {}
impl Directionality for Asymmetric {}

pub struct Distances<T> {
    pub data: Vec<u32>,
    pub dimension: usize,
    directionality: std::marker::PhantomData<T>,
}

impl Distances<Symmetric> {
    pub fn new_symmetric_from_data(distance_data: Vec<u32>, dimension: usize) -> Self {
        Self {
            data: distance_data,
            dimension,
            directionality: std::marker::PhantomData,
        }
    }

    pub fn new_symmetric_from_dimension(dimension: usize) -> Self {
        let size = (dimension as usize * (dimension as usize - 1)) / 2;
        Self {
            data: Vec::with_capacity(size),
            dimension,
            directionality: std::marker::PhantomData,
        }
    }

    pub fn get_distance(&self, from: usize, to: usize) -> u32 {
        let index = get_lower_triangle_matrix_entry(from, to);
        self.data[index]
    }
}

impl Distances<Asymmetric> {
    pub fn new_asymmetric_from_data(distance_data: Vec<u32>, dimension: usize) -> Self {
        Self {
            data: distance_data,
            dimension,
            directionality: std::marker::PhantomData,
        }
    }

    pub fn new_asymmetric_from_dimension(dimension: usize) -> Self {
        let size = dimension * dimension;
        Self {
            data: Vec::with_capacity(size),
            dimension,
            directionality: std::marker::PhantomData,
        }
    }

    pub fn get_distance(&self, from: usize, to: usize) -> u32 {
        let index = get_row_major_matrix_entry(from, to, self.dimension);
        self.data[index]
    }
}

#[inline]
/// Computes the index in a vec-flattened lower-(left-)triangular matrix.
pub fn get_lower_triangle_matrix_entry(row: usize, column: usize) -> usize {
    // TODO: Check if upper triangular matrix would be faster for some reason
    let (row, column) = if row > column {
        (row, column)
    } else {
        (column, row)
    };
    (row * (row + 1)) / 2 + column
}

#[inline]
/// Computes the index in a vec-flattened matrix in row-major order.
pub fn get_row_major_matrix_entry(row: usize, column: usize, dimension: usize) -> usize {
    row * dimension + column
}
