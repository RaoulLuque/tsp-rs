use memchr::memchr;
use memmap2::Mmap;
use thiserror::Error;
use tsp_core::{
    instance::InstanceMetadata,
    tsp_lib_spec::{
        DisplayDataType, EdgeDataFormat, EdgeWeightFormat, EdgeWeightType, NodeCoordType,
        ProblemType, TSPDataKeyword,
    },
};

use crate::{
    FileContent, ParserError,
    metadata::metadata_builder::{InstanceMetadataBuilder, InstanceMetadataBuilderError},
};

pub mod metadata_builder;

#[derive(Error, Debug)]
pub enum MetaDataParseError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Invalid keyword in this line: {0}")]
    InvalidKeyword(String),
    #[error("Invalid (problem) TYPE value: {0}")]
    InvalidProblemType(String),
    #[error("Invalid DIMENSION value: {0}")]
    InvalidDimension(String),
    #[error("Invalid CAPACITY value: {0}")]
    InvalidCapacity(String),
    #[error("Invalid EDGE_WEIGHT_TYPE value: {0}")]
    InvalidEdgeWeightType(String),
    #[error("Invalid EDGE_WEIGHT_FORMAT value: {0}")]
    InvalidEdgeWeightFormat(String),
    #[error("Invalid EDGE_DATA_FORMAT value: {0}")]
    InvalidEdgeDataFormat(String),
    #[error("Invalid NODE_COORD_TYPE value: {0}")]
    InvalidNodeCoordType(String),
    #[error("Invalid DISPLAY_DATA_TYPE value: {0}")]
    InvalidDisplayDataType(String),
    #[error(transparent)]
    InstanceMetadataBuilderError(#[from] InstanceMetadataBuilderError),
}

/// Parses the metadata section of a TSP instance file.
///
/// Returns a tuple containing the parsed `InstanceMetadata`, the first encountered
/// `TSPDataKeyword`, and a reference to the remaining lines iterator starting from the data section
/// (the line after the first data keyword).
pub fn parse_metadata(
    file_content: &FileContent,
    index_in_map: &mut usize,
) -> Result<(InstanceMetadata, TSPDataKeyword), ParserError> {
    let mut metadata_builder = InstanceMetadataBuilder::new();
    let data_keyword = loop {
        let Some(index_newline) = memchr(b'\n', &file_content[*index_in_map..]) else {
            return Err(
                MetaDataParseError::InvalidInput("Unexpected end of file".to_string()).into(),
            );
        };

        let line = unsafe {
            std::str::from_utf8_unchecked(
                &file_content[*index_in_map..*index_in_map + index_newline],
            )
        };
        // println!("Parsing line: {}", line);

        // Move the index to the start of the next line (+1 for the newline character)
        *index_in_map += index_newline + 1;

        match parse_specification_or_data_keyword(line, &mut metadata_builder)? {
            None => {
                // The specification keyword has been added to the builder inside
                // parse_specification_or_data_keyword
            }
            Some(data_keyword) => {
                // Reached data section, break the loop
                break data_keyword;
            }
        }
    };
    let metadata = metadata_builder.build()?;

    Ok((metadata, data_keyword))
}

fn parse_specification_or_data_keyword(
    line: &str,
    metadata_builder: &mut InstanceMetadataBuilder,
) -> Result<Option<TSPDataKeyword>, ParserError> {
    let mut parts = line.splitn(2, ':');
    match (parts.next(), parts.next()) {
        // Hot path
        (Some(k), Some(v)) => {
            parse_specification(k.trim(), v.trim(), metadata_builder)?;
            Ok(None)
        }
        // Cold path(s)
        (Some(k), None) => Ok(Some(parse_data_keyword(k.trim())?)),
        _ => Err(MetaDataParseError::InvalidKeyword(line.to_string()).into()),
    }
}

fn parse_specification(
    keyword: &str,
    value: &str,
    metadata_builder: &mut InstanceMetadataBuilder,
) -> Result<(), ParserError> {
    match keyword {
        "NAME" => {
            metadata_builder.name_mut(value.to_string());
            Ok(())
        }
        "TYPE" => {
            metadata_builder.problem_type_mut(parse_problem_type(value)?);
            Ok(())
        }
        "COMMENT" => {
            metadata_builder.comment_mut(value.to_string());
            Ok(())
        }
        "DIMENSION" => {
            metadata_builder.dimension_mut(
                value
                    .parse()
                    .map_err(|_| MetaDataParseError::InvalidDimension(value.to_string()))?,
            );
            Ok(())
        }
        "CAPACITY" => {
            metadata_builder.capacity_mut(
                value
                    .parse()
                    .map_err(|_| MetaDataParseError::InvalidCapacity(value.to_string()))?,
            );
            Ok(())
        }
        "EDGE_WEIGHT_TYPE" => {
            metadata_builder.edge_weight_type_mut(parse_edge_weight_type(value)?);
            Ok(())
        }
        "EDGE_WEIGHT_FORMAT" => {
            metadata_builder.edge_weight_format_mut(parse_edge_weight_format(value)?);
            Ok(())
        }
        "EDGE_DATA_FORMAT" => {
            metadata_builder.edge_data_format_mut(parse_edge_data_format(value)?);
            Ok(())
        }
        "NODE_COORD_TYPE" => {
            metadata_builder.node_coord_type_mut(parse_node_coord_type(value)?);
            Ok(())
        }
        "DISPLAY_DATA_TYPE" => {
            metadata_builder.display_data_type_mut(parse_display_data_type(value)?);
            Ok(())
        }
        _ => Err(MetaDataParseError::InvalidKeyword(keyword.to_string()).into()),
    }
}

fn parse_data_keyword(input: &str) -> Result<TSPDataKeyword, ParserError> {
    match input {
        "NODE_COORD_SECTION" => Ok(TSPDataKeyword::NODE_COORD_SECTION),
        "DEPOT_SECTION" => Ok(TSPDataKeyword::DEPOT_SECTION),
        "DEMAND_SECTION" => Ok(TSPDataKeyword::DEMAND_SECTION),
        "EDGE_DATA_SECTION" => Ok(TSPDataKeyword::EDGE_DATA_SECTION),
        "FIXED_EDGES_SECTION" => todo!("Fixed edges sections are not supported yet"),
        "DISPLAY_DATA_SECTION" => Ok(TSPDataKeyword::DISPLAY_DATA_SECTION),
        "TOUR_SECTION" => Ok(TSPDataKeyword::TOUR_SECTION),
        "EDGE_WEIGHT_SECTION" => Ok(TSPDataKeyword::EDGE_WEIGHT_SECTION),
        _ => Err(MetaDataParseError::InvalidKeyword(input.to_string()).into()),
    }
}

fn parse_problem_type(input: &str) -> Result<ProblemType, ParserError> {
    match input {
        "TSP" => Ok(ProblemType::TSP),
        "ATSP" => Ok(ProblemType::ATSP),
        "SOP" => Ok(ProblemType::SOP),
        "HCP" => Ok(ProblemType::HCP),
        "TOUR" => Ok(ProblemType::TOUR),
        _ => Err(MetaDataParseError::InvalidProblemType(input.to_string()).into()),
    }
}

fn parse_edge_weight_type(input: &str) -> Result<EdgeWeightType, ParserError> {
    match input {
        "EXPLICIT" => Ok(EdgeWeightType::EXPLICIT),
        "EUC_2D" => Ok(EdgeWeightType::EUC_2D),
        "EUC_3D" => Ok(EdgeWeightType::EUC_3D),
        "MAX_2D" => Ok(EdgeWeightType::MAX_2D),
        "MAX_3D" => Ok(EdgeWeightType::MAX_3D),
        "MAN_2D" => Ok(EdgeWeightType::MAN_2D),
        "MAN_3D" => Ok(EdgeWeightType::MAN_3D),
        "CEIL_2D" => Ok(EdgeWeightType::CEIL_2D),
        "GEO" => Ok(EdgeWeightType::GEO),
        "ATT" => Ok(EdgeWeightType::ATT),
        "XRAY1" => Ok(EdgeWeightType::XRAY1),
        "XRAY2" => Ok(EdgeWeightType::XRAY2),
        "SPECIAL" => Ok(EdgeWeightType::SPECIAL),
        _ => Err(MetaDataParseError::InvalidEdgeWeightType(input.to_string()).into()),
    }
}

fn parse_edge_weight_format(input: &str) -> Result<EdgeWeightFormat, ParserError> {
    match input {
        "FUNCTION" => Ok(EdgeWeightFormat::FUNCTION),
        "FULL_MATRIX" => Ok(EdgeWeightFormat::FULL_MATRIX),
        "UPPER_ROW" => Ok(EdgeWeightFormat::UPPER_ROW),
        "LOWER_ROW" => Ok(EdgeWeightFormat::LOWER_ROW),
        "UPPER_DIAG_ROW" => Ok(EdgeWeightFormat::UPPER_DIAG_ROW),
        "LOWER_DIAG_ROW" => Ok(EdgeWeightFormat::LOWER_DIAG_ROW),
        "UPPER_COL" => Ok(EdgeWeightFormat::UPPER_COL),
        "LOWER_COL" => Ok(EdgeWeightFormat::LOWER_COL),
        "UPPER_DIAG_COL" => Ok(EdgeWeightFormat::UPPER_DIAG_COL),
        "LOWER_DIAG_COL" => Ok(EdgeWeightFormat::LOWER_DIAG_COL),
        _ => Err(MetaDataParseError::InvalidEdgeWeightFormat(input.to_string()).into()),
    }
}

fn parse_edge_data_format(input: &str) -> Result<EdgeDataFormat, ParserError> {
    match input {
        "EDGE_LIST" => Ok(EdgeDataFormat::EDGE_LIST),
        "ADJ_LIST" => Ok(EdgeDataFormat::ADJ_LIST),
        _ => Err(MetaDataParseError::InvalidEdgeDataFormat(input.to_string()).into()),
    }
}

fn parse_node_coord_type(input: &str) -> Result<NodeCoordType, ParserError> {
    match input {
        "TWOD_COORDS" => Ok(NodeCoordType::TWOD_COORDS),
        "THREED_COORDS" => Ok(NodeCoordType::THREED_COORDS),
        "NO_COORDS" => Ok(NodeCoordType::NO_COORDS),
        _ => Err(MetaDataParseError::InvalidNodeCoordType(input.to_string()).into()),
    }
}

fn parse_display_data_type(input: &str) -> Result<DisplayDataType, ParserError> {
    match input {
        "COORD_DISPLAY" => Ok(DisplayDataType::COORD_DISPLAY),
        "TWOD_DISPLAY" => Ok(DisplayDataType::TWOD_DISPLAY),
        "NO_DISPLAY" => Ok(DisplayDataType::NO_DISPLAY),
        _ => Err(MetaDataParseError::InvalidDisplayDataType(input.to_string()).into()),
    }
}
