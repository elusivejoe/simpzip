use crate::unpacker::algorithms::Decompressor;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom};

fn is_bit_set(byte: u8, n_bit: u8) -> bool {
    let mask = 1u8 << (n_bit - 1);

    (byte & mask) > 0
}

fn print_bytes(bytes: &[u8]) {
    for byte in bytes {
        print!("{:b} ", byte);
    }
}

fn print_first_block_header(bytes: &[u8]) {
    let bit_1 = is_bit_set(bytes[0], 8);
    let bit_2 = is_bit_set(bytes[0], 7);
    let bit_3 = is_bit_set(bytes[0], 6);

    println!();

    if bit_1 {
        println!(":: Last block in the stream");
    } else {
        println!(":: There are more blocks in the stream");
    }

    if !bit_2 && !bit_3 {
        println!(":: Literal (raw) section");
    } else if !bit_2 && bit_3 {
        println!(":: Static Huffman compressed block");
    } else if bit_2 && !bit_3 {
        println!(":: Compressed block with Huffman table")
    } else {
        println!(":: Reserved")
    }
}

pub struct DeflateDecompressor;
impl Decompressor for DeflateDecompressor {
    fn decompress(
        &self,
        reader: &mut BufReader<File>,
        _writer: &mut BufWriter<&mut File>,
        data_pos: &u64,
        data_len: &u64,
    ) -> std::io::Result<()> {
        reader.seek(SeekFrom::Start(*data_pos))?;

        const CHUNK_SIZE: usize = 1024 * 1024;
        let mut bytes_left = *data_len;

        println!(">> Compressed data >>");

        while bytes_left > 0 {
            let next_bytes = std::cmp::min(bytes_left, CHUNK_SIZE as u64);

            if next_bytes == CHUNK_SIZE as u64 {
                let mut bytes = [0u8; CHUNK_SIZE];
                reader.read_exact(&mut bytes)?;

                print_bytes(&bytes);
                print_first_block_header(&bytes);
            } else {
                let mut bytes = vec![0u8; next_bytes as usize];
                reader.read_exact(&mut bytes)?;

                print_bytes(&bytes);
                print_first_block_header(&bytes);
            }

            bytes_left -= next_bytes;
        }

        println!("<< Compressed data <<");

        Ok(())
    }
}
