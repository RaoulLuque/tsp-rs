use std::{fs::File, path::Path};

use memmap2::{Advice, Mmap};
use thiserror::Error;
use tsp_core::instance::TSPSymInstance;

use crate::{
    distance_container::ParseFromTSPLib,
    data_section::parse_data_sections,
    metadata::{MetaDataParseError, parse_metadata},
};

pub mod distance_container;
pub mod data_section;
pub mod metadata;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    MetaDataParsing(#[from] MetaDataParseError),
}

pub fn parse_tsp_instance<DistanceContainer: ParseFromTSPLib>(
    instance_path: impl AsRef<Path>,
) -> Result<TSPSymInstance<DistanceContainer>, ParserError> {
    // Safety: This is the only point at which we access the file, so the file should not be
    // modified otherwise.
    let mmap = unsafe { Mmap::map(&File::open(instance_path)?)? };
    mmap.advise(Advice::Sequential)?;
    let mut index_in_map = 0;

    let (metadata, data_keyword) = parse_metadata(&mmap, &mut index_in_map)?;

    let data =
        parse_data_sections::<DistanceContainer>(&mmap, &mut index_in_map, data_keyword, &metadata);

    Ok(TSPSymInstance::new(data, metadata))
}
