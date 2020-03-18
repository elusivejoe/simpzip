use std::env;
use std::fs::{File};
use std::io::{BufReader, Read, Seek, SeekFrom};

mod read_convenience;
use read_convenience::{read_to, stream_current_position, stream_length};

mod zip_feature_versions;
mod zip_signatures;

mod zip_structs;
use zip_structs::{LocalFileHeader, CentralDirFileHeader, EndOfCentralDir};

mod args_parser;
use args_parser::{parse_args};

fn read_signature<T: Read>(reader: &mut T) -> std::io::Result<u32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;

    read_to::<u32>(&bytes, &mut 0)
}

fn read_local_file_header<T: Read + Seek>(reader: &mut T) -> std::io::Result<LocalFileHeader> {
    const FIRST_CHUNK_SIZE: usize = 34;

    let mut bytes= [0; FIRST_CHUNK_SIZE];

    reader.read_exact(&mut bytes)?;

    let mut offset = 0;

    let ver_to_extract;
    let file_name_len;

    let result = LocalFileHeader {
        version_to_extract:  {
            ver_to_extract = read_to::<u16>(&bytes, &mut offset)?;

            ver_to_extract
        },

        general_bit_flag:   read_to::<u16>(&bytes, &mut offset)?,
        compression_method: read_to::<u16>(&bytes, &mut offset)?,
        last_mod_file_time: read_to::<u16>(&bytes, &mut offset)?,
        last_mod_file_date: read_to::<u16>(&bytes, &mut offset)?,
        crc_32:             read_to::<u32>(&bytes, &mut offset)?,

        compressed_size: {
            if ver_to_extract >= zip_feature_versions::ZIP64 {
                read_to::<u64>(&bytes, &mut offset)?
            } else {
                read_to::<u32>(&bytes, &mut offset)? as u64
            }
        },

        uncompressed_size: {
            if ver_to_extract >= zip_feature_versions::ZIP64 {
                read_to::<u64>(&bytes, &mut offset)?
            } else {
                read_to::<u32>(&bytes, &mut offset)? as u64
            }
        },

        file_name_length: {
            file_name_len = read_to::<u16>(&bytes, &mut offset)?;

            file_name_len
        },

        extra_field_length: read_to::<u16>(&bytes, &mut offset)?,

        file_name: {
            if ver_to_extract < zip_feature_versions::ZIP64 {
                reader.seek(SeekFrom::Current(-8))?;
            }

            let mut bytes = vec![0u8; file_name_len as usize];
            reader.read_exact(&mut bytes)?;

            String::from_utf8(bytes).unwrap() //TODO: convert to I/O Error
        }
    };

    reader.seek(SeekFrom::Current(result.extra_field_length as i64))?;

    Ok(result)
}

fn read_central_dir_file_header<T: Read + Seek>(reader: &mut T) -> std::io::Result<CentralDirFileHeader> {
    const FIRST_CHUNK_SIZE: usize = 50;

    let mut bytes = [0; FIRST_CHUNK_SIZE];

    reader.read_exact(&mut bytes)?;

    let mut offset = 0;

    let ver_to_extract;
    let file_name_len;
    let extra_field_len;
    let file_comment_len;

    let result = CentralDirFileHeader {
        version_made_by: read_to::<u16>(&bytes, &mut offset)?,
        version_to_extract:  {
            ver_to_extract = read_to::<u16>(&bytes, &mut offset)?;

            ver_to_extract
        },

        general_bit_flag:   read_to::<u16>(&bytes, &mut offset)?,
        compression_method: read_to::<u16>(&bytes, &mut offset)?,
        last_mod_file_time: read_to::<u16>(&bytes, &mut offset)?,
        last_mod_file_date: read_to::<u16>(&bytes, &mut offset)?,
        crc_32:             read_to::<u32>(&bytes, &mut offset)?,

        compressed_size: {
            if ver_to_extract >= zip_feature_versions::ZIP64 {
                read_to::<u64>(&bytes, &mut offset)?
            } else {
                read_to::<u32>(&bytes, &mut offset)? as u64
            }
        },

        uncompressed_size: {
            if ver_to_extract >= zip_feature_versions::ZIP64 {
                read_to::<u64>(&bytes, &mut offset)?
            } else {
                read_to::<u32>(&bytes, &mut offset)? as u64
            }
        },

        file_name_length: {
            if ver_to_extract < zip_feature_versions::ZIP64 as u16 {
                reader.seek(SeekFrom::Current(-8))?;
            }

            file_name_len = read_to::<u16>(&bytes, &mut offset)?;

            file_name_len
        },

        extra_field_length: {
            extra_field_len = read_to::<u16>(&bytes, &mut offset)?;

            extra_field_len
        },

        file_comment_length: {
            file_comment_len = read_to::<u16>(&bytes, &mut offset)?;

            file_comment_len
        },

        disk_number_start:       read_to::<u16>(&bytes, &mut offset)?,
        internal_file_attribs:   read_to::<u16>(&bytes, &mut offset)?,
        external_file_attribs:   read_to::<u32>(&bytes, &mut offset)?,
        local_header_rel_offset: read_to::<u32>(&bytes, &mut offset)?,

        file_name: {
            let mut bytes = vec![0u8; file_name_len as usize];
            reader.read_exact(&mut bytes)?;

            //skipping extra field
            reader.seek(SeekFrom::Current(extra_field_len as i64))?;

            String::from_utf8(bytes).unwrap() //TODO: convert to I/O Error
        },

        file_comment: {
            let mut bytes = vec![0u8; file_comment_len as usize];
            reader.read_exact(&mut bytes)?;

            String::from_utf8(bytes).unwrap() //TODO: convert to I/O Error
        }
    };

    Ok(result)
}

fn read_read_end_of_central_dir<T: Read>(reader: &mut T) -> std::io::Result<EndOfCentralDir> {
    const FIRST_CHUNK_SIZE: usize = 18;

    let mut bytes = [0; FIRST_CHUNK_SIZE];

    reader.read_exact(&mut bytes)?;

    let mut offset = 0;

    let zip_file_comment_len;

    let result = EndOfCentralDir {
        number_of_this_disk:                       read_to::<u16>(&bytes, &mut offset)?,
        number_of_disk_with_start_central_dir:     read_to::<u16>(&bytes, &mut offset)?,
        total_entries_in_central_dir_on_this_disk: read_to::<u16>(&bytes, &mut offset)?,
        total_entries_in_central_dir:              read_to::<u16>(&bytes, &mut offset)?,
        central_dir_size:                          read_to::<u32>(&bytes, &mut offset)?,
        central_dir_offset_from_starting_disk_num: read_to::<u32>(&bytes, &mut offset)?,

        zip_file_comment_length: {
            zip_file_comment_len = read_to::<u16>(&bytes, &mut offset)?;

            zip_file_comment_len
        },

        zip_file_comment: {
            let mut bytes = vec![0u8; zip_file_comment_len as usize];
            reader.read_exact(&mut bytes)?;

            String::from_utf8(bytes).unwrap() //TODO: convert to I/O Error
        }
    };

    Ok(result)
}

fn read_file_data<T: Read>(reader: &mut T, data_size: &u64) -> std::io::Result<()> {
    const CHUNK_SIZE: usize = 1024;
    let mut bytes_left = *data_size;

    while bytes_left > 0 {
        let next_bytes = std::cmp::min(bytes_left, CHUNK_SIZE as u64);

        if next_bytes == CHUNK_SIZE as u64 {
            let mut bytes = [0u8; CHUNK_SIZE];
            reader.read_exact(&mut bytes)?;
        } else {
            let mut bytes = vec![0u8; next_bytes as usize];
            reader.read_exact(&mut bytes)?;
        }

        bytes_left -= next_bytes;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args = parse_args(&env::args().collect()).unwrap();

    println!("Source ZIP: {}\n", args.path);

    let zip_file = File::open(&args.path)?;
    let mut buf_reader = BufReader::new(zip_file);
    let stream_len = stream_length(&mut buf_reader)?;

    while stream_current_position(&mut buf_reader)? < stream_len {
        let signature = read_signature(&mut buf_reader)?;

        if signature == zip_signatures::SIGNATURE_FILE_HEADER {
            match read_local_file_header(&mut buf_reader) {
                Ok(local_file_header) => {
                    println!("{:#?}", local_file_header);
                    read_file_data(&mut buf_reader, &local_file_header.compressed_size)?;
                },
                Err(err) => {
                    println!("Error reading local file header. Reason: {}", err.to_string());
                }
            }
        }

        if signature == zip_signatures::SIGNATURE_CENTRAL_DIR_HEADER {
            match read_central_dir_file_header(&mut buf_reader) {
                Ok(central_dir_file_header) => {
                    println!("{:#?}", central_dir_file_header);
                },
                Err(err) => {
                    println!("Error reading central dir file header. Reason: {}", err.to_string());
                }
            }
        }

        if signature == zip_signatures::SIGNATURE_CENTRAL_DIR_END {
            match read_read_end_of_central_dir(&mut buf_reader) {
                Ok(end_of_central_dir) => {
                    println!("{:#?}", end_of_central_dir);
                },
                Err(err) => {
                    println!("Error reading end of central dir. Reason: {}", err.to_string());
                }
            }
        }

        println!("\nRead {} out of {}\n", stream_current_position(&mut buf_reader)?, stream_len);
    }

    Ok(())
}
