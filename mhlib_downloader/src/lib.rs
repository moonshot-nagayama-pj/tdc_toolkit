use async_http_range_reader::{AsyncHttpRangeReader, CheckSupportMethod};
use async_zip::tokio::read::seek::ZipFileReader;
use hex_literal::hex;
use http::header::HeaderMap;
use once_cell::sync::Lazy;
use reqwest::Client;
use sha3::{Digest, Sha3_512};
use std::collections::HashMap;
use tokio::fs::File;
use tokio::io;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tokio_util::compat::TokioAsyncReadCompatExt;
use url::Url;

pub const DEFAULT_MHLIB_VERSION: &str = "3.1";

// This is the SHA3-512 hash of the Linux shared library.
// This can be generated using openssl, for instance:
// openssl dgst -sha3-512 mhlib.so
pub static MHLIB_VERSION_SHA3_512: Lazy<HashMap<&str, [u8; 64]>> = Lazy::new(|| {
    HashMap::from([
    ("3.1", hex!("dcc2c30a4054cbdecd9085797097742e5a43110d19acb848ffb4fd50006b500d0d04d604e6449ad61c24d361efe9655df797520a58fcd1f37023297c8b1d3525")),
    ])
});

// Based on test cases from async_http_range_reader
pub async fn download() {
    let (mut range, _) = AsyncHttpRangeReader::new(
        Client::new(),
        Url::parse("https://www.picoquant.com/dl_software/MultiHarp150/MultiHarp150_160_V3_1.zip").unwrap(),
        CheckSupportMethod::Head,
        HeaderMap::default(),
    )
        .await
        .expect("Could not retrieve the range for this file - is the URL incorrect or is it possible the server does not support HTTP range requests?");

    // Make sure we have read the last couple of bytes, as zip files
    // have a central directory record at the end
    // TODO this will fail on files of less than 8192 bytes
    range.prefetch(range.len() - 8192..range.len()).await;

    let mut zip_reader = ZipFileReader::new(range.compat()).await.unwrap();

    // Prefetch the data for the tarball
    let entry_position = zip_reader.file().entries().iter()
        .position(|entry| entry.entry().filename().as_str().unwrap() == "MHLib v3.1.0.0/Linux/MHLib_v3.1.0.0_64bit.tar.gz")
        .unwrap();
    let stored_entry = zip_reader.file().entries().get(entry_position).unwrap();
    let entry = stored_entry.entry();
    let offset = stored_entry.header_offset();

    // Get the size of the entry plus the header + size of the filename. We should also actually
    // include bytes for the extra fields but we don't have that information.
    let size =
        entry.compressed_size() + 30 + entry.filename().as_bytes().len() as u64;

    // The zip archive uses as BufReader which reads in chunks of 8192. To ensure we prefetch
    // enough data we round the size up to the nearest multiple of the buffer size.
    let buffer_size = 8192;
    let size = ((size + buffer_size - 1) / buffer_size) * buffer_size;

    // Fetch the bytes from the zip archive that contain the requested file.
    zip_reader
        .inner_mut()
        .get_mut()
        .prefetch(offset..offset + size as u64)
        .await;

    // Read the contents of the tarball into a file
    let mut file = File::create("tarball.tar.gz").await.unwrap();
    let mut zip_entry_reader = zip_reader
        .reader_with_entry(entry_position)
        .await
        .unwrap()
        .compat();
    let _ = io::copy(&mut zip_entry_reader, &mut file).await.unwrap();
}

pub fn validate_hash() {
    let mut hasher = Sha3_512::new();
    hasher.update(b"abc");
    let hash = hasher.finalize();
    println!("test hash: {:?}", hash);
}
