use crate::unpacker::algorithms::Decompressor;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};

pub struct StoreDecompressor;
impl Decompressor for StoreDecompressor {
    fn decompress(
        &self,
        reader: &mut BufReader<File>,
        writer: &mut BufWriter<&mut File>,
        data_pos: &u64,
        data_len: &u64,
    ) -> std::io::Result<()> {
        reader.seek(SeekFrom::Start(*data_pos))?;

        const CHUNK_SIZE: usize = 1024 * 1024;
        let mut bytes_left = *data_len;

        //TODO: verify checksum
        while bytes_left > 0 {
            let next_bytes = std::cmp::min(bytes_left, CHUNK_SIZE as u64);

            if next_bytes == CHUNK_SIZE as u64 {
                let mut bytes = [0u8; CHUNK_SIZE];
                reader.read_exact(&mut bytes)?;

                writer.write(&bytes)?;
            } else {
                //TODO: too many allocations here; consider passing reusable external buffer
                let mut bytes = vec![0u8; next_bytes as usize];
                reader.read_exact(&mut bytes)?;

                writer.write(&bytes)?;
            }

            bytes_left -= next_bytes;
        }

        Ok(())
    }
}
