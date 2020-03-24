use std::io::{Seek, SeekFrom};

pub fn current_position<T: Seek>(stream: &mut T) -> std::io::Result<u64> {
    stream.seek(SeekFrom::Current(0))
}

pub fn length<T: Seek>(stream: &mut T) -> std::io::Result<u64> {
    let old_pos = current_position(stream)?;
    let len = stream.seek(SeekFrom::End(0))?;
    stream.seek(SeekFrom::Start(old_pos))?;

    Ok(len)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{BufReader, Read};

    use crate::stream_utils::stream_nav::{current_position, length};

    #[test]
    fn stream_navigation() {
        let test_file = File::open("test-data/streams/streams_0.txt");

        let mut reader = match test_file {
            Ok(file) => BufReader::new(file),
            Err(_) => {
                assert!(false, "Couldn't open test data file.");
                BufReader::new(File::open("").unwrap())
            }
        };

        let stream_len = length(&mut reader).unwrap_or(0);

        assert_eq!(stream_len, 8);

        match reader.read_exact(&mut [0u8; 4]) {
            Ok(_) => {}
            Err(_) => {
                assert!(false, "Couldn't navigate over the stream.");
            }
        }

        let stream_current_pos = current_position(&mut reader).unwrap_or(0);

        assert_eq!(stream_current_pos, 4);
    }
}
