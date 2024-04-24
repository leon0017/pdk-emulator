use snafu::prelude::*;
use std::{
    fs::{self, File},
    io::Read,
};

#[derive(Debug, Snafu)]
pub enum ProgramError {
    #[snafu(display("Failed to open file"))]
    OpenFile { source: std::io::Error },
    #[snafu(display("Failed to read metadata"))]
    ReadMetadata { source: std::io::Error },
    #[snafu(display("Buffer overflow"))]
    BufferOverflow { source: std::io::Error },
}

pub fn read_program(file_path: &str) -> Result<Vec<u8>, ProgramError> {
    let mut file = File::open(&file_path).context(OpenFileSnafu)?;
    let metadata = fs::metadata(&file_path).context(ReadMetadataSnafu)?;
    let mut buffer = vec![0u8; metadata.len() as usize];
    file.read(&mut buffer).context(BufferOverflowSnafu)?;

    Ok(buffer)
}
