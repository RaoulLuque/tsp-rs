use memchr::memchr;
use tsp_core::instance::{distance::Distance, matrix::TriangularMatrix};

use crate::{FileContent, data_section::loop_over_lines};

pub(super) fn parse_dists_from_full_matrix(
    data: &FileContent,
    index_in_map: &mut usize,
    dimension: usize,
) -> TriangularMatrix<Distance> {
    let mut res_data = Vec::with_capacity((dimension * (dimension + 1)) / 2);
    let mut current_line_number = 0;

    loop_over_lines(
        data,
        index_in_map,
        &mut (&mut res_data, &mut current_line_number),
        |line: &str, (res, current_line_number): &mut (&mut Vec<Distance>, &mut usize)| {
            parse_full_line_into_distances(res, line, current_line_number, dimension)
        },
        |(_, current_line_number): &(&mut Vec<Distance>, &mut usize)| {
            **current_line_number >= dimension
        },
    );
    // Add Distance from last node to itself as that's not parsed by parse_line_into_distances
    res_data.push(Distance(0));

    TriangularMatrix::new(res_data, dimension)
}

/// Parses distances from a single line of the explicit full matrix section,
/// adding them to the provided matrix data vector.
///
/// The function updates the line number to keep track of how many lines have been parsed.
/// The line number should start from 0 for the first line of distances.
///
/// For the last line, for performance reasons it parses only up to dimension - 1 distances, that
/// is, the distance from the last node to the last node is not added to the matrix data (hint: it's
/// always zero).
fn parse_full_line_into_distances(
    matrix_data: &mut Vec<Distance>,
    line: &str,
    line_number: &mut usize,
    dimension: usize,
) {
    let mut start_index_in_line = 0;
    let mut num_nodes_parsed = 0;

    if *line_number < dimension - 1 {
        while num_nodes_parsed <= *line_number {
            let space_index = memchr(b' ', &line.as_bytes()[start_index_in_line..]).expect(
                "There should be more distances in the explicit matrix section and thus more \
                 spaces.",
            );
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
            num_nodes_parsed += 1;
        }
    } else {
        // We are parsing the last line of the matrix
        while num_nodes_parsed < *line_number {
            let space_index = memchr(b' ', &line.as_bytes()[start_index_in_line..]).expect(
                "There should be more distances in the explicit matrix section and thus more \
                 spaces.",
            );
            if space_index == 0 {
                start_index_in_line += 1;
                continue;
            }
            let end_index = start_index_in_line + space_index;
            let distance_str = &line[start_index_in_line..end_index];
            let distance = parse_distance_from_str(distance_str);
            matrix_data.push(distance);
            start_index_in_line = end_index + 1;
            num_nodes_parsed += 1;
        }
    }

    *line_number += 1;
}

fn parse_distance_from_str(s: &str) -> Distance {
    let parsed_value: i32 = s
        .parse()
        .expect("Entries in explicit matrix section should be valid integers.");
    Distance(parsed_value)
}
