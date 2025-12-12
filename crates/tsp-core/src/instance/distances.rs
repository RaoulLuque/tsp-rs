#[derive(Debug, Clone)]
pub struct DistancesSymmetric {
    pub data: Vec<u32>,
    pub dimension: usize,
}

impl DistancesSymmetric {
    pub fn new_from_data(distance_data: Vec<u32>, dimension: usize) -> Self {
        Self {
            data: distance_data,
            dimension,
        }
    }

    pub fn new_from_dimension(dimension: usize) -> Self {
        let size = (dimension * (dimension - 1)) / 2;
        Self {
            data: Vec::with_capacity(size),
            dimension,
        }
    }

    #[inline(always)]
    pub fn get_distance(&self, from: usize, to: usize) -> u32 {
        let index = get_lower_triangle_matrix_entry(from, to);
        self.data[index]
    }
}

#[inline(always)]
/// Computes the index in a vec-flattened lower-(left-)triangular matrix.
pub fn get_lower_triangle_matrix_entry(row: usize, column: usize) -> usize {
    // TODO: Check if upper triangular matrix would be faster for some reason
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

#[inline(always)]
/// Computes the row and column from a linear index of a lower-triangular matrix.
/// Returns (row, column) with column >= row.
pub fn get_row_col_from_index(index: usize) -> (usize, usize) {
    // We solve the quadratic equation: c^2 + c - 2k = 0
    // The simplified integer formula for the column is: floor((sqrt(8k + 1) - 1) / 2)
    
    // Using integer square root (available in Rust 1.67+) to avoid float precision issues
    let column = ((8 * index + 1).isqrt() - 1) / 2;
    
    // The row is simply the remainder after subtracting the triangular base
    let triangular_base = (column * (column + 1)) / 2;
    let row = index - triangular_base;

    (row, column)
}