use memchr::memchr;
use tsp_core::instance::{
    distance::Distance,
    matrix::SquareMatrix,
    node::Node,
};

use crate::{FileContent, data_section::loop_over_lines};

pub(super) fn parse_dists_from_full_matrix(
    data: &FileContent,
    index_in_map: &mut usize,
    dimension: usize,
) -> SquareMatrix<Distance> {
    let mut res_data = Vec::with_capacity(dimension * dimension);
    let mut current_row = 0;
    let mut current_column = 0;

    loop_over_lines(
        data,
        index_in_map,
        &mut (&mut res_data, &mut current_row, &mut current_column),
        |line: &str, (res, current_row, current_column): &mut (&mut Vec<Distance>, &mut usize, &mut usize)| {
            parse_full_matrix_line(res, line, current_row, current_column, dimension)
        },
        |(_, current_row, _): &(&mut Vec<Distance>, &mut usize, &mut usize)| {
            **current_row >= dimension
        },
    );

    SquareMatrix::new(res_data, dimension)
}

pub(super) fn parse_dists_from_lower_row(
    data: &FileContent,
    index_in_map: &mut usize,
    dimension: usize,
    diagonal_entry_present: bool,
) -> SquareMatrix<Distance> {
    let mut res = SquareMatrix::new_from_dimension_with_value(dimension, Distance(0));
    let mut current_row = 0;
    let mut current_column = 0;

    loop_over_lines(
        data,
        index_in_map,
        &mut (&mut res, &mut current_row, &mut current_column),
        |line: &str,
         (res, current_row, current_column): &mut (
            &mut SquareMatrix<Distance>,
            &mut usize,
            &mut usize,
        )| {
            parse_lower_triangular_line(
                res,
                line,
                current_row,
                current_column,
                diagonal_entry_present,
            )
        },
        |(_, current_row, _): &(&mut SquareMatrix<Distance>, &mut usize, &mut usize)| {
            **current_row >= dimension
        },
    );

    res
}

pub(super) fn parse_dists_from_upper_row(
    data: &FileContent,
    index_in_map: &mut usize,
    dimension: usize,
    diagonal_entry_present: bool,
) -> SquareMatrix<Distance> {
    let mut res = SquareMatrix::new_from_dimension_with_value(dimension, Distance(0));
    let mut current_row = 0;
    let mut current_column = 0;

    loop_over_lines(
        data,
        index_in_map,
        &mut (&mut res, &mut current_row, &mut current_column),
        |line: &str,
         (res, current_row, current_column): &mut (
            &mut SquareMatrix<Distance>,
            &mut usize,
            &mut usize,
        )| {
            parse_upper_triangular_line(
                res,
                line,
                current_row,
                current_column,
                dimension,
                diagonal_entry_present,
            )
        },
        |(_, current_row, _): &(&mut SquareMatrix<Distance>, &mut usize, &mut usize)| {
            **current_row >= dimension - 1
        },
    );

    res
}

/// Parses distances from a single line of the explicit full matrix section,
/// adding them to the provided matrix data vector.
/// Full matrix line in this context means that the line contains distances
/// from node i to all other nodes (0 to dimension - 1).
#[inline]
fn parse_full_matrix_line(
    matrix_data: &mut Vec<Distance>,
    line: &str,
    row: &mut usize,
    column: &mut usize,
    dimension: usize,
) {
    let mut start_index_in_line = 0;

    while start_index_in_line < line.len() {
        let space_index = match memchr(b' ', &line.as_bytes()[start_index_in_line..]) {
            Some(index) => index,
            None => line.len() - start_index_in_line,
        };
        // Unfortunately the TSPLIB file spec does not specify how distances are separated (in
        // particular whether multiple spaces can appear between distances), so we
        // have to handle that case
        if space_index == 0 {
            start_index_in_line += 1;
            continue;
        }
        let end_index = start_index_in_line + space_index;
        let distance_str = &line[start_index_in_line..end_index];
        let distance = parse_distance_from_str(distance_str);
        matrix_data.push(distance);

        start_index_in_line = end_index + 1;
        *column += 1;

        if *column >= dimension {
            *row += 1;
            *column = 0;
        }
    }
}

/// Parses distances from a single line of the explicit full matrix section,
/// adding them to the provided matrix.
/// Lower diagonal line in this context means that for line i, the line starts with the distances
/// from node i to nodes 0, 1, ..., i (inclusive).
///
/// The function updates the row and column indices to keep track of the current position
/// in the matrix being parsed (0-indexed).
#[inline]
fn parse_lower_triangular_line(
    matrix_data: &mut SquareMatrix<Distance>,
    line: &str,
    row: &mut usize,
    column: &mut usize,
    diagonal_entry_present: bool,
) {
    let mut start_index_in_line = 0;

    while start_index_in_line < line.len() {
        if !diagonal_entry_present && *column == *row {
            *row += 1;
            *column = 0;
            // We don't need to explicitly add the distance from the node to itself as 0, since the
            // matrix is already initialized with all zeros.
        }

        let space_index = match memchr(b' ', &line.as_bytes()[start_index_in_line..]) {
            Some(index) => index,
            None => line.len() - start_index_in_line,
        };
        // Unfortunately the TSPLIB file spec does not specify how distances are separated (in
        // particular whether multiple spaces can appear between distances), so we
        // have to handle that case
        if space_index == 0 {
            start_index_in_line += 1;
            continue;
        }
        let end_index = start_index_in_line + space_index;
        let distance_str = &line[start_index_in_line..end_index];
        let distance = parse_distance_from_str(distance_str);
        matrix_data.set_data_symmetric(Node(*row), Node(*column), distance);

        start_index_in_line = end_index + 1;
        *column += 1;

        if *column > *row {
            *row += 1;
            *column = 0;
        }
    }
}

/// Parses distances from a single line of the explicit full matrix section,
/// adding them to the provided matrix.
/// Upper triangular line in this context means that for line i, the line starts with the distances
/// from node i to nodes i + 1, i + 2, ..., dimension - 1 (inclusive).
///
/// The function updates the row and column indices to keep track of the current position
/// in the matrix being parsed (0-indexed).
#[inline]
fn parse_upper_triangular_line(
    matrix_data: &mut SquareMatrix<Distance>,
    line: &str,
    row: &mut usize,
    column: &mut usize,
    dimension: usize,
    diagonal_entry_present: bool,
) {
    let mut start_index_in_line = 0;

    while start_index_in_line < line.len() {
        if !diagonal_entry_present && *column == *row {
            *column += 1;
            // If the diagonal entry is not present, we need to add the distance from the node
            // to itself as 0. However, since the default value in the matrix is already 0,
            // we do not need to explicitly set it here.
        }

        let space_index = match memchr(b' ', &line.as_bytes()[start_index_in_line..]) {
            Some(index) => index,
            None => line.len() - start_index_in_line,
        };
        // Unfortunately the TSPLIB file spec does not specify how distances are separated (in
        // particular whether multiple spaces can appear between distances), so we
        // have to handle that case
        if space_index == 0 {
            start_index_in_line += 1;
            continue;
        }
        let end_index = start_index_in_line + space_index;
        let distance_str = &line[start_index_in_line..end_index];
        let distance: Distance = parse_distance_from_str(distance_str);
        matrix_data.set_data_symmetric(Node(*row), Node(*column), distance);

        start_index_in_line = end_index + 1;
        *column += 1;

        if *column >= dimension {
            *row += 1;
            *column = *row;
        }
    }
}

#[inline]
fn parse_distance_from_str(s: &str) -> Distance {
    let parsed_value: i32 = s.parse().expect(&format!(
        "Entries in explicit matrix section should be valid integers. Trying to parse '{}'",
        s
    ));
    Distance(parsed_value)
}
