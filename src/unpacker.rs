mod algorithms;

use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;

use crate::stream_utils::{byte_readers, stream_nav};
use crate::zip::structs::{CentralDirFileHeader, EndOfCentralDir, LocalFileHeader};
use crate::zip::{compression_methods, feature_versions, signatures};

fn read_signature<T: Read>(reader: &mut T) -> std::io::Result<u32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;

    byte_readers::read_to::<u32>(&bytes, &mut 0)
}

fn read_local_file_header<T: Read + Seek>(reader: &mut T) -> std::io::Result<LocalFileHeader> {
    const FIRST_CHUNK_SIZE: usize = 34;

    let mut bytes = [0; FIRST_CHUNK_SIZE];

    reader.read_exact(&mut bytes)?;

    let mut offset = 0;

    let ver_to_extract;
    let file_name_len;

    let result = LocalFileHeader {
        version_to_extract: {
            ver_to_extract = byte_readers::read_to::<u16>(&bytes, &mut offset)?;

            ver_to_extract
        },

        general_bit_flag: byte_readers::read_to::<u16>(&bytes, &mut offset)?,
        compression_method: byte_readers::read_to::<u16>(&bytes, &mut offset)?,
        last_mod_file_time: byte_readers::read_to::<u16>(&bytes, &mut offset)?,
        last_mod_file_date: byte_readers::read_to::<u16>(&bytes, &mut offset)?,
        crc_32: byte_readers::read_to::<u32>(&bytes, &mut offset)?,

        compressed_size: {
            if ver_to_extract >= feature_versions::ZIP64 {
                byte_readers::read_to::<u64>(&bytes, &mut offset)?
            } else {
                byte_readers::read_to::<u32>(&bytes, &mut offset)? as u64
            }
        },

        uncompressed_size: {
            if ver_to_extract >= feature_versions::ZIP64 {
                byte_readers::read_to::<u64>(&bytes, &mut offset)?
            } else {
                byte_readers::read_to::<u32>(&bytes, &mut offset)? as u64
            }
        },

        file_name_length: {
            file_name_len = byte_readers::read_to::<u16>(&bytes, &mut offset)?;

            file_name_len
        },

        extra_field_length: byte_readers::read_to::<u16>(&bytes, &mut offset)?,

        file_name: {
            if ver_to_extract < feature_versions::ZIP64 {
                reader.seek(SeekFrom::Current(-8))?;
            }

            let mut bytes = vec![0u8; file_name_len as usize];
            reader.read_exact(&mut bytes)?;

            String::from_utf8(bytes).unwrap() //TODO: convert to I/O Error
        },
    };

    reader.seek(SeekFrom::Current(result.extra_field_length as i64))?;

    Ok(result)
}

fn read_central_dir_file_header<T: Read + Seek>(
    reader: &mut T,
) -> std::io::Result<CentralDirFileHeader> {
    const FIRST_CHUNK_SIZE: usize = 50;

    let mut bytes = [0; FIRST_CHUNK_SIZE];

    reader.read_exact(&mut bytes)?;

    let mut offset = 0;

    let ver_to_extract;
    let file_name_len;
    let extra_field_len;
    let file_comment_len;

    let result = CentralDirFileHeader {
        version_made_by: byte_readers::read_to::<u16>(&bytes, &mut offset)?,
        version_to_extract: {
            ver_to_extract = byte_readers::read_to::<u16>(&bytes, &mut offset)?;

            ver_to_extract
        },

        general_bit_flag: byte_readers::read_to::<u16>(&bytes, &mut offset)?,
        compression_method: byte_readers::read_to::<u16>(&bytes, &mut offset)?,
        last_mod_file_time: byte_readers::read_to::<u16>(&bytes, &mut offset)?,
        last_mod_file_date: byte_readers::read_to::<u16>(&bytes, &mut offset)?,
        crc_32: byte_readers::read_to::<u32>(&bytes, &mut offset)?,

        compressed_size: {
            if ver_to_extract >= feature_versions::ZIP64 {
                byte_readers::read_to::<u64>(&bytes, &mut offset)?
            } else {
                byte_readers::read_to::<u32>(&bytes, &mut offset)? as u64
            }
        },

        uncompressed_size: {
            if ver_to_extract >= feature_versions::ZIP64 {
                byte_readers::read_to::<u64>(&bytes, &mut offset)?
            } else {
                byte_readers::read_to::<u32>(&bytes, &mut offset)? as u64
            }
        },

        file_name_length: {
            if ver_to_extract < feature_versions::ZIP64 as u16 {
                reader.seek(SeekFrom::Current(-8))?;
            }

            file_name_len = byte_readers::read_to::<u16>(&bytes, &mut offset)?;

            file_name_len
        },

        extra_field_length: {
            extra_field_len = byte_readers::read_to::<u16>(&bytes, &mut offset)?;

            extra_field_len
        },

        file_comment_length: {
            file_comment_len = byte_readers::read_to::<u16>(&bytes, &mut offset)?;

            file_comment_len
        },

        disk_number_start: byte_readers::read_to::<u16>(&bytes, &mut offset)?,
        internal_file_attribs: byte_readers::read_to::<u16>(&bytes, &mut offset)?,
        external_file_attribs: byte_readers::read_to::<u32>(&bytes, &mut offset)?,
        local_header_rel_offset: byte_readers::read_to::<u32>(&bytes, &mut offset)?,

        file_name: {
            let mut bytes = vec![0u8; file_name_len as usize];
            reader.read_exact(&mut bytes)?;

            //TODO: use extra field; skipping it for now
            reader.seek(SeekFrom::Current(extra_field_len as i64))?;

            String::from_utf8(bytes).unwrap() //TODO: convert to I/O Error
        },

        file_comment: {
            let mut bytes = vec![0u8; file_comment_len as usize];
            reader.read_exact(&mut bytes)?;

            String::from_utf8(bytes).unwrap() //TODO: convert to I/O Error
        },
    };

    Ok(result)
}

fn read_end_of_central_dir<T: Read>(reader: &mut T) -> std::io::Result<EndOfCentralDir> {
    const FIRST_CHUNK_SIZE: usize = 18;

    let mut bytes = [0; FIRST_CHUNK_SIZE];

    reader.read_exact(&mut bytes)?;

    let mut offset = 0;

    let zip_file_comment_len;

    let result = EndOfCentralDir {
        number_of_this_disk: byte_readers::read_to::<u16>(&bytes, &mut offset)?,
        number_of_disk_with_start_central_dir: byte_readers::read_to::<u16>(&bytes, &mut offset)?,
        total_entries_in_central_dir_on_this_disk: byte_readers::read_to::<u16>(
            &bytes,
            &mut offset,
        )?,
        total_entries_in_central_dir: byte_readers::read_to::<u16>(&bytes, &mut offset)?,
        central_dir_size: byte_readers::read_to::<u32>(&bytes, &mut offset)?,
        central_dir_offset_from_starting_disk_num: byte_readers::read_to::<u32>(
            &bytes,
            &mut offset,
        )?,

        zip_file_comment_length: {
            zip_file_comment_len = byte_readers::read_to::<u16>(&bytes, &mut offset)?;

            zip_file_comment_len
        },

        zip_file_comment: {
            let mut bytes = vec![0u8; zip_file_comment_len as usize];
            reader.read_exact(&mut bytes)?;

            String::from_utf8(bytes).unwrap() //TODO: convert to I/O Error
        },
    };

    Ok(result)
}

fn is_folder(local_file_header: &LocalFileHeader) -> bool {
    local_file_header.compression_method == compression_methods::STORE
        && local_file_header.version_to_extract == feature_versions::DIR_OR_DEFLATE
}

pub fn unpack_archive(src_file: &Path, out_dir: &Path) -> std::io::Result<()> {
    if let Some(_) = out_dir.read_dir()?.next() {
        return Err(std::io::Error::new(
            ErrorKind::Other,
            "Output dir is not empty.",
        ));
    };

    let src_file = File::open(src_file)?;
    let mut src_file_reader = BufReader::new(src_file);

    let stream_len = stream_nav::length(&mut src_file_reader)?;

    while stream_nav::current_position(&mut src_file_reader)? < stream_len {
        let signature = read_signature(&mut src_file_reader)?;

        if signature == signatures::SIGNATURE_FILE_HEADER {
            match read_local_file_header(&mut src_file_reader) {
                Ok(local_file_header) => {
                    if is_folder(&local_file_header) {
                        std::fs::create_dir(out_dir.join(local_file_header.file_name))?;
                    } else {
                        let mut out_file =
                            File::create(out_dir.join(&local_file_header.file_name))?;
                        let mut buf_writer = BufWriter::new(&mut out_file);
                        let stream_pos = stream_nav::current_position(&mut src_file_reader)?;

                        //TODO: extremely inefficient on a large amount of small files
                        algorithms::decompressor(&local_file_header.compression_method)
                            .unwrap()
                            .decompress(
                                &mut src_file_reader,
                                &mut buf_writer,
                                &stream_pos,
                                &local_file_header.compressed_size,
                            )?;
                    }
                }
                Err(err) => {
                    println!(
                        "Error reading local file header. Reason: {}",
                        err.to_string()
                    );
                }
            }
        }

        if signature == signatures::SIGNATURE_CENTRAL_DIR_HEADER {
            match read_central_dir_file_header(&mut src_file_reader) {
                Ok(_central_dir_file_header) => {}
                Err(err) => {
                    println!(
                        "Error reading central dir file header. Reason: {}",
                        err.to_string()
                    );
                }
            }
        }

        if signature == signatures::SIGNATURE_CENTRAL_DIR_END {
            match read_end_of_central_dir(&mut src_file_reader) {
                Ok(_end_of_central_dir) => {}
                Err(err) => {
                    println!(
                        "Error reading end of central dir. Reason: {}",
                        err.to_string()
                    );
                }
            }
        }
    }

    println!(
        "Read {} out of {}",
        stream_nav::current_position(&mut src_file_reader)?,
        stream_len
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use dir_diff;
    use std::path::Path;

    #[test]
    fn unpack_store_0() {
        let out_folder = Path::new("test-data/unpack_store_0/actual");

        if out_folder.exists() && !out_folder.is_dir() {
            assert!(false, "Output dir is not a dir.");
        } else if out_folder.exists() {
            std::fs::remove_dir_all(out_folder).unwrap();
        }

        std::fs::create_dir(out_folder).unwrap();

        super::unpack_archive(Path::new("test-data/unpack_store_0/input.zip"), out_folder).unwrap();

        assert!(!dir_diff::is_different(
            out_folder,
            Path::new("test-data/unpack_store_0/expected")
        )
        .unwrap());

        std::fs::remove_dir_all(out_folder).unwrap();
    }

    #[test]
    fn unpack_deflate_normal_dict32kb_word32() {
        unimplemented!("test is broken until Deflate is implemented");

        /*let out_folder = Path::new("test-data/unpack_deflate/normal_dict32kb_word32/actual");

        if out_folder.exists() && !out_folder.is_dir() {
            assert!(false, "Output dir is not a dir.");
        } else if out_folder.exists() {
            std::fs::remove_dir_all(out_folder).unwrap();
        }

        std::fs::create_dir(out_folder).unwrap();

        super::unpack_archive(
            Path::new("test-data/unpack_deflate/normal_dict32kb_word32/input.zip"),
            out_folder,
        )
        .unwrap();

        assert!(!dir_diff::is_different(
            out_folder,
            Path::new("test-data/unpack_deflate/normal_dict32kb_word32/expected")
        )
        .unwrap());

        std::fs::remove_dir_all(out_folder).unwrap();*/
    }
}
