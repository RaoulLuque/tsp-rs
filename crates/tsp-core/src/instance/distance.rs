pub trait DistanceMatrix {
    fn get_distance(&self, from: usize, to: usize) -> u32;
    fn dimension(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct DistanceMatrixSymmetric {
    pub data: Vec<u32>,
    pub dimension: usize,
}

impl DistanceMatrixSymmetric {
    pub fn new_from_data(distance_data: Vec<u32>, dimension: usize) -> Self {
        assert_eq!(distance_data.len(), dimension * (dimension + 1) / 2);
        Self {
            data: distance_data,
            dimension,
        }
    }

    pub fn new_from_dimension_with_value(dimension: usize, value: u32) -> Self {
        let size = (dimension * (dimension + 1)) / 2;
        Self {
            data: vec![value; size],
            dimension,
        }
    }

    pub fn slow_new_from_distance_function(
        dimension: usize,
        mut distance_function: impl FnMut(usize, usize) -> u32,
    ) -> Self {
        let mut res = DistanceMatrixSymmetric::new_from_dimension_with_value(dimension, 0);
        for row in 0..dimension {
            for column in 0..row {
                let distance = distance_function(row, column);
                res.set_distance(row, column, distance);
            }
        }
        res
    }

    #[inline(always)]
    pub fn get_distance(&self, from: usize, to: usize) -> u32 {
        let index = get_lower_triangle_matrix_entry(from, to);
        self.data[index]
    }

    #[inline(always)]
    pub fn set_distance(&mut self, from: usize, to: usize, distance: u32) {
        let index = get_lower_triangle_matrix_entry(from, to);
        self.data[index] = distance;
    }

    pub fn restrict_to_first_n<'a>(&'a self, n: usize) -> RestrictedDistanceMatrixSymmetric<'a> {
        RestrictedDistanceMatrixSymmetric {
            data: &self.data[0..(n * (n - 1)) / 2],
            dimension: n,
        }
    }
}

impl DistanceMatrix for DistanceMatrixSymmetric {
    fn get_distance(&self, from: usize, to: usize) -> u32 {
        self.get_distance(from, to)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

pub struct RestrictedDistanceMatrixSymmetric<'a> {
    pub data: &'a [u32],
    pub dimension: usize,
}

impl<'a> RestrictedDistanceMatrixSymmetric<'a> {
    #[inline(always)]
    pub fn get_distance(&self, from: usize, to: usize) -> u32 {
        let index = get_lower_triangle_matrix_entry(from, to);
        self.data[index]
    }
}

impl<'a> DistanceMatrix for RestrictedDistanceMatrixSymmetric<'a> {
    fn get_distance(&self, from: usize, to: usize) -> u32 {
        self.get_distance(from, to)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

#[inline(always)]
/// Computes the index in a vec-flattened lower-(left-)triangular matrix.
pub fn get_lower_triangle_matrix_entry(row: usize, column: usize) -> usize {
    if row > column {
        get_lower_triangle_matrix_entry_row_bigger(row, column)
    } else {
        get_lower_triangle_matrix_entry_column_bigger(row, column)
    }
}

#[inline(always)]
/// Computes the index in a vec-flattened lower-(left-)triangular matrix assuming row >= column.
pub fn get_lower_triangle_matrix_entry_row_bigger(row: usize, column: usize) -> usize {
    (row * (row + 1)) / 2 + column
}

#[inline(always)]
/// Computes the index in a vec-flattened lower-(left-)triangular matrix assuming column >= row.
pub fn get_lower_triangle_matrix_entry_column_bigger(row: usize, column: usize) -> usize {
    (column * (column + 1)) / 2 + row
}
