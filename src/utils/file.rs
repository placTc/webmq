use std::{fs::metadata, io::Read, path::Path};

use crate::core::errors::WebMQError;

pub fn get_file_length(file: &Path) -> Result<usize, WebMQError> {
    let file_meta = metadata(file);
    let file_meta = match file_meta {
        Ok(meta) => meta,
        Err(e) => {
            return Err(WebMQError::File(format!(
                "Couldn't get length of file {}: {}",
                file.to_string_lossy(),
                e
            )));
        }
    };

    Ok(file_meta.len() as usize)
}

pub fn get_file_buffer(file: &Path) -> Result<Vec<u8>, WebMQError> {
    let cert_file_length = match get_file_length(file) {
        Ok(len) => len,
        Err(e) => return Err(WebMQError::Config(e.to_string())),
    };

    let mut buffer = vec![0; cert_file_length];

    match std::fs::File::open(file).map(|mut file| {
        let _ = file.read(&mut buffer);
        buffer
    }) {
        Err(e) => Err(WebMQError::File(format!(
            "Couldn't open file {}: {}",
            file.to_string_lossy(),
            e
        ))),
        Ok(vec) => Ok(vec),
    }
}
