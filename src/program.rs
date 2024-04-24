use snafu::prelude::*;
use std::{
    fs::{self, File},
    io::Read,
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to open file"))]
    OpenFile { source: std::io::Error },
    #[snafu(display("Failed to read metadata"))]
    ReadMetadata { source: std::io::Error },
    #[snafu(display("Buffer overflow"))]
    BufferOverflow { source: std::io::Error },
    #[snafu(display("File size u64 to usize conversion failed"))]
    FileSizeConversion { source: std::num::TryFromIntError },
}

pub fn read(file_path: &str) -> Result<Vec<u8>, Error> {
    let mut file = File::open(file_path).context(OpenFileSnafu)?;
    let metadata = fs::metadata(file_path).context(ReadMetadataSnafu)?;
    let mut buffer = vec![0u8; metadata.len().try_into().context(FileSizeConversionSnafu)?];
    file.read(&mut buffer).context(BufferOverflowSnafu)?;

    Ok(buffer)
}
