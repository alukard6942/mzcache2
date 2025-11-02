use endian_trait::Endian;

use crate::{
    error::{MzError, MzResult},
    utils::{read_struct_buff, Timestamp},
};

use std::{
    fmt::{Debug, Display, UpperHex},
    fs::File,
    io::Cursor,
    mem::{size_of, MaybeUninit},
};

use crate::utils::read_struct;

#[derive(Debug)]
pub struct IndexFile {
    pub header: Header,
    // todo: hasmap with the hash as a hash
    pub records: Vec<Record>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Endian)]
struct Header {
    version: u32,
    // POSIX timestampt in UTC
    last_modification: Timestamp,
    is_dirty: u32,
    kb_writen: u32,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Hash([u8; 20]);

fn hex_char_to_number(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        'A'..='F' => Some(c as u8 - b'A' + 10),
        'a'..='f' => Some(c as u8 - b'a' + 10),
        _ => None,
    }
}

impl From<&String> for Hash {
    fn from(value: &String) -> Self {
        let hashsize = size_of::<Hash>();

        assert!(value.as_bytes().len() == hashsize * 2);

        let mut buf = MaybeUninit::<[u8; size_of::<Hash>()]>::uninit();

        for i in 0..hashsize {
            let ten = hex_char_to_number(value.as_bytes()[i * 2] as char).unwrap();
            let one = hex_char_to_number(value.as_bytes()[i * 2 + 1] as char).unwrap();

            let hex = ten * 16 + one;

            unsafe {
                buf.assume_init_mut()[i] = hex;
            }
        }
        read_struct_buff(unsafe { &buf.assume_init() }).unwrap()
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        for n in self.0 {
            let f = format!("{:02X}", n);
            s.push_str(&f);
        }

        // write!(f,"{}", String::from_utf8_lossy(&self.0))

        // f.debug_tuple("Hash").field(&self.0).finish()

        write!(f, "{}", s)
    }
}

impl Endian for Hash {
    fn to_be(self) -> Self {
        self
    }
    fn to_le(self) -> Self {
        self
    }
    fn from_be(self) -> Self {
        self
    }
    fn from_le(self) -> Self {
        self
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, Endian)]
pub struct Record {
    pub hash: Hash,

    // determins when will a cachefile be deleted
    pub frecency: u32,
    pub origin_attr_hash: u64,
    pub on_start_time: u16,
    pub on_stop_time: u16,
    pub content_type: u8,

    /*
     *    1000 0000 0000 0000 0000 0000 0000 0000 : initialized
     *    0100 0000 0000 0000 0000 0000 0000 0000 : anonymous
     *    0010 0000 0000 0000 0000 0000 0000 0000 : removed
     *    0001 0000 0000 0000 0000 0000 0000 0000 : dirty
     *    0000 1000 0000 0000 0000 0000 0000 0000 : fresh
     *    0000 0100 0000 0000 0000 0000 0000 0000 : pinned
     *    0000 0010 0000 0000 0000 0000 0000 0000 : has cached alt data
     *    0000 0001 0000 0000 0000 0000 0000 0000 : reserved
     *    0000 0000 1111 1111 1111 1111 1111 1111 : file size (in kB)
     */
    pub flags: u32,
}

pub fn read_index_file(b: &mut [u8]) -> MzResult<IndexFile> {
    let mut cursor = Cursor::new(b);

    let header: Header = match read_struct(&mut cursor) {
        Some(it) => it,
        None => return Err(MzError::MissingHeader),
    };
    let mut records = Vec::new();

    while let Some(r) = read_struct(&mut cursor) {
        records.push(r);
    }

    Ok(IndexFile { header, records })
}

#[cfg(test)]
mod tests {
    use std::{fs::File, mem::size_of};

    use crate::utils::read_struct;

    use super::*;

    #[test]
    fn sizetest() {
        assert_eq!(size_of::<Header>(), 16);
        assert_eq!(size_of::<Record>(), 41);
    }

    #[test]
    fn header_test() {
        let path = "cache2/index";
        let mut f = File::open(path.to_string()).unwrap();

        let h: Header = read_struct(&mut f).unwrap();

        let t = format!("{}", h.last_modification);

        assert_eq!(h.version, 10);
        assert_eq!(t, "2023-02-27T19:45:18");
        assert_eq!(h.is_dirty, 1);
    }

    #[test]
    fn record_test() {
        let path = "cache2/index";
        let mut f = File::open(path.to_string()).unwrap();

        let _: Header = read_struct(&mut f).unwrap();

        // fuck you for not leting me declere the type as the part of the let Some statment
        while let Some(record) = {
            let tmp: Option<Record> = read_struct(&mut f);
            tmp
        } {
            print!("{:#?}", record);
            assert!(record.on_start_time <= record.on_stop_time);
        }
    }
}
