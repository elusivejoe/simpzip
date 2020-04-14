use std::fs::File;
use std::io::{BufReader, BufWriter};

use crate::zip::compression_methods;

mod deflate;
mod store;

pub trait Decompressor {
    fn decompress(
        &self,
        reader: &mut BufReader<File>,
        writer: &mut BufWriter<&mut File>,
        data_pos: &u64,
        data_len: &u64,
    ) -> std::io::Result<()>;
}

pub fn decompressor(compression_method: &u16) -> Result<Box<dyn Decompressor>, &'static str> {
    match compression_method {
        &compression_methods::STORE => Ok(Box::new(store::StoreDecompressor)),
        &compression_methods::DEFLATE => Ok(Box::new(deflate::DeflateDecompressor)),
        _ => Err("Unknown compression method."),
    }
}
