use std::io::{Read, Write};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};

pub fn compress_gzip_data(original_data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(original_data)?;
    encoder.finish() // finish() 返回 Result<Vec<u8>, io::Error>
}

pub fn decompress_gzip_data(compressed_data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = GzDecoder::new(compressed_data);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;
    Ok(decompressed_data)
}
