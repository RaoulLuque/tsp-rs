use std::{fs::File, ops::Deref, path::Path};

use memmap2::{Advice, Mmap};
use thiserror::Error;
use tsp_core::instance::TSPSymInstance;

use crate::{
    data_section::parse_data_sections,
    distance_container::ParseFromTSPLib,
    metadata::{MetaDataParseError, parse_metadata},
};

pub mod data_section;
pub mod distance_container;
pub mod metadata;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    MetaDataParsing(#[from] MetaDataParseError),
}

pub struct FileContent {
    #[cfg(not(feature = "miri"))]
    data: Mmap,

    #[cfg(feature = "miri")]
    data: Vec<u8>,
}

pub fn parse_tsp_instance<DistanceContainer: ParseFromTSPLib>(
    instance_path: impl AsRef<Path>,
) -> Result<TSPSymInstance<DistanceContainer>, ParserError> {
    let file_content = FileContent::new(instance_path)?;
    let mut index_in_map = 0;

    let (metadata, data_keyword) = parse_metadata(&file_content, &mut index_in_map)?;

    let data =
        parse_data_sections::<DistanceContainer>(&file_content, &mut index_in_map, data_keyword, &metadata);

    Ok(TSPSymInstance::new(data, metadata))
}

impl FileContent {
    pub fn new(instance_path: impl AsRef<Path>) -> Result<Self, ParserError> {
        #[cfg(feature = "miri")]
        {
            let data = std::fs::read(instance_path)?;
            Ok(FileContent { data })
        }
        #[cfg(not(feature = "miri"))]
        {
            // Safety: This is the only point at which we access the file, so the file should
            // not be modified otherwise.
            let mmap = unsafe { Mmap::map(&File::open(instance_path)?)? };
            mmap.advise(Advice::Sequential)?;
            Ok(FileContent { data: mmap })
        }
    }
}

impl Deref for FileContent {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        #[cfg(feature = "miri")]
        {
            &self.data
        }
        #[cfg(not(feature = "miri"))]
        {
            &self.data[..]
        }
    }
}
