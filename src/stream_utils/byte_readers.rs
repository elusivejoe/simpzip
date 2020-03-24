use std::convert::TryInto;
use std::io::{Error, ErrorKind};
use std::mem::size_of;

pub trait FromLeBytes {
    fn from(bytes: &[u8]) -> Self;
}

impl FromLeBytes for u16 {
    fn from(bytes: &[u8]) -> u16 {
        u16::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl FromLeBytes for u32 {
    fn from(bytes: &[u8]) -> u32 {
        u32::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl FromLeBytes for u64 {
    fn from(bytes: &[u8]) -> u64 {
        u64::from_le_bytes(bytes.try_into().unwrap())
    }
}

pub fn read_to<T: FromLeBytes>(bytes: &[u8], offset: &mut usize) -> std::io::Result<T> {
    let begin = *offset;
    *offset += size_of::<T>();
    let end = *offset;

    match bytes[begin..end].try_into() {
        Ok(slice) => Ok(T::from(slice)),
        Err(err) => Err(Error::new(ErrorKind::Other, err.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use crate::stream_utils::byte_readers::read_to;

    #[test]
    fn read_to_u16() {
        let bytes: [u8; 8] = [82, 35, 155, 154, 255, 255, 0, 54];

        let mut offset: usize = 0;

        let word1 = read_to::<u16>(&bytes, &mut offset).unwrap_or(0);
        let word2 = read_to::<u16>(&bytes, &mut offset).unwrap_or(0);
        let word3 = read_to::<u16>(&bytes, &mut offset).unwrap_or(0);
        let word4 = read_to::<u16>(&bytes, &mut offset).unwrap_or(0);

        assert_eq!(word1, 9042u16);
        assert_eq!(word2, 39579u16);
        assert_eq!(word3, 65535u16);
        assert_eq!(word4, 13824u16);
    }

    #[test]
    fn read_to_u32() {
        let bytes: [u8; 8] = [178, 65, 222, 252, 255, 255, 255, 255];

        let mut offset: usize = 0;

        let dword1 = read_to::<u32>(&bytes, &mut offset).unwrap_or(0);
        let dword2 = read_to::<u32>(&bytes, &mut offset).unwrap_or(0);

        assert_eq!(dword1, 4242424242u32);
        assert_eq!(dword2, 4294967295u32);
    }

    #[test]
    fn read_to_u64() {
        let bytes: [u8; 16] = [
            7, 255, 181, 37, 199, 37, 40, 13, 255, 255, 255, 255, 255, 255, 255, 255,
        ];

        let mut offset: usize = 0;

        let qword1 = read_to::<u64>(&bytes, &mut offset).unwrap_or(0);
        let qword2 = read_to::<u64>(&bytes, &mut offset).unwrap_or(0);

        assert_eq!(qword1, 948049258822893319);
        assert_eq!(qword2, 18446744073709551615u64);
    }
}
