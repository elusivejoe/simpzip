//TODO: implement a builder instead of using public fields
//TODO: move structs into their own separate files

#[derive(Debug)]
pub struct LocalFileHeader {
    pub version_to_extract: u16,
    pub general_bit_flag: u16,
    pub compression_method: u16,
    pub last_mod_file_time: u16,
    pub last_mod_file_date: u16,
    pub crc_32: u32,
    pub compressed_size: u64,   //can be 4 byte
    pub uncompressed_size: u64, //can be 4 byte
    pub file_name_length: u16,
    pub extra_field_length: u16,
    pub file_name: String,
    //extra_field: skip for now
}

#[derive(Debug)]
pub struct CentralDirFileHeader {
    pub version_made_by: u16,
    pub version_to_extract: u16,
    pub general_bit_flag: u16,
    pub compression_method: u16,
    pub last_mod_file_time: u16,
    pub last_mod_file_date: u16,
    pub crc_32: u32,
    pub compressed_size: u64,   //can be 4 byte
    pub uncompressed_size: u64, //can be 4 byte
    pub file_name_length: u16,
    pub extra_field_length: u16,
    pub file_comment_length: u16,
    pub disk_number_start: u16,
    pub internal_file_attribs: u16,
    pub external_file_attribs: u32,
    pub local_header_rel_offset: u32,
    pub file_name: String,
    //extra_field: skip for now
    pub file_comment: String,
}

#[derive(Debug)]
pub struct EndOfCentralDir {
    pub number_of_this_disk: u16,
    pub number_of_disk_with_start_central_dir: u16,
    pub total_entries_in_central_dir_on_this_disk: u16,
    pub total_entries_in_central_dir: u16,
    pub central_dir_size: u32,
    pub central_dir_offset_from_starting_disk_num: u32,
    pub zip_file_comment_length: u16,
    pub zip_file_comment: String,
}
