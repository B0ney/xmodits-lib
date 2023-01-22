use std::{
    io::{self, BufReader, Cursor, Read, Seek, SeekFrom},
    vec,
};

use crate::{parser::{
    bitflag::BitFlag,
    magic::{self, bad_magic_non_consume, magic, magic_non_consume},
    read_str,
}, interface::Sample};
// use std
pub trait Data: Read + Seek + Send + Sync {
    fn size(&self) -> Option<u64>;
}

impl<T: AsRef<[u8]> + Send + Sync> Data for Cursor<T> {
    fn size(&self) -> Option<u64> {
        Some(self.get_ref().as_ref().len() as u64)
    }
}

impl Data for std::fs::File {
    fn size(&self) -> Option<u64> {
        match self.metadata() {
            Ok(x) => Some(x.len()),
            _ => None,
        }
    }
}

impl<T: Data> Data for BufReader<T> {
    fn size(&self) -> Option<u64> {
        // self.stream_len()
        None
    }
}

// impl<T: Data> ByteReader for T {
//     fn read_word(&mut self) -> io::Result<[u8; 2]> {
//         let mut buf = [0u8; 2];
//         self.read_exact(&mut buf)?;
//         Ok(buf)
//     }

//     fn read_dword(&mut self) -> io::Result<[u8; 4]> {
//         let mut buf = [0u8; 4];
//         self.read_exact(&mut buf)?;
//         Ok(buf)
//     }

//     fn read_byte(&mut self) -> io::Result<u8> {
//         let mut buf = [0u8; 1];
//         self.read_exact(&mut buf)?;
//         Ok(buf[0])
//     }
//     fn skip_bytes(&mut self, bytes: i64) -> io::Result<()> {
//         self.seek(SeekFrom::Current(bytes)).map(|_| ())
//     }

//     fn read_bytes_boxed_slice(&mut self, bytes: usize) -> io::Result<Box<[u8]>> {
//         let mut buf = vec![0; bytes];
//         self.read_exact(&mut buf)?;
//         Ok(buf.into_boxed_slice())
//     }
// }

impl From<Box<dyn Data>> for DataStream {
    fn from(inner: Box<dyn Data>) -> Self {
        Self { inner }
    }
}

struct DataStream {
    inner: Box<dyn Data>,
}
impl DataStream {
    fn new(data: Box<dyn Data>) -> Self {
        Self { inner: data }
    }
}

// impl <'b, R: Data> ByteReader for R {
//     fn read_byte(&mut self) -> io::Result<u8> {
//         (*self).read_byte()
//     }

//     fn read_word(&mut self) -> io::Result<[u8; 2]> {
//         (*self).read_word()
//     }

//     fn read_dword(&mut self) -> io::Result<[u8; 4]> {
//         (*self).read_dword()
//     }

//     fn skip_bytes(&mut self, bytes: i64) -> io::Result<()> {
//         (*self).skip_bytes(bytes)
//     }

//     fn read_bytes_boxed_slice(&mut self, bytes: usize) -> io::Result<Box<[u8]>> {
//         (*self).read_bytes_boxed_slice(bytes)
//     }
// }

pub trait ByteReader {
    fn read_byte(&mut self) -> io::Result<u8>;
    fn read_word(&mut self) -> io::Result<[u8; 2]>;
    fn read_dword(&mut self) -> io::Result<[u8; 4]>;
    fn read_u8(&mut self) -> io::Result<u8> {
        self.read_byte()
    }
    fn read_u16_le(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.read_word()?))
    }
    fn read_u16_be(&mut self) -> io::Result<u16> {
        Ok(u16::from_be_bytes(self.read_word()?))
    }
    fn read_u32_le(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.read_dword()?))
    }
    fn read_u32_be(&mut self) -> io::Result<u32> {
        Ok(u32::from_be_bytes(self.read_dword()?))
    }
    fn skip_bytes(&mut self, bytes: i64) -> io::Result<()>;
    fn set_seek_pos(&mut self, offset: u64) -> io::Result<()>;

    fn read_bytes_boxed_slice(&mut self, bytes: usize) -> io::Result<Box<[u8]>>;
    fn size(&self) -> Option<u64>;
}
// enum Writy {
//     Vec(),
//     File()
// // }

impl<T: Data> ByteReader for T {
    fn read_word(&mut self) -> io::Result<[u8; 2]> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_dword(&mut self) -> io::Result<[u8; 4]> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_byte(&mut self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }
    fn skip_bytes(&mut self, bytes: i64) -> io::Result<()> {
        self.seek(SeekFrom::Current(bytes)).map(|_| ())
    }

    fn read_bytes_boxed_slice(&mut self, bytes: usize) -> io::Result<Box<[u8]>> {
        let mut buf = vec![0; bytes];
        self.read_exact(&mut buf)?;
        Ok(buf.into_boxed_slice())
    }

    fn set_seek_pos(&mut self, offset: u64) -> io::Result<()> {
        self.seek(SeekFrom::Start(offset)).map(|_| ())
    }

    fn size(&self) -> Option<u64> {
        self.size()
    }
}
mod flag {
    pub const STEREO: u8 = 1 << 3;
}
#[test]
fn g() {
    let mut buf = Cursor::new(b"IMPMSpace War\0\0\0\0\0\0\0\0\0\0\0");
    // DataStream::new(Box::new(buf));
    // let mut buf = BufReader::new(std::fs::File::open("path").unwrap());
    // let header = buf.read_bytes_boxed_slice(2).unwrap();
    // header.into_vec();
    // let buf = DataStream::new(Box::new(buf));
    // buf.skip_bytes(-1);
    // let s = (&mut buf).read_bytes_boxed_slice(20).unwrap();
    // dbg!(String::from_utf8_lossy(&s));
    dbg!(bad_magic_non_consume(&mut buf, b"ziRCON"));
    // let  b = &buf.read_bytes_boxed_slice(20).unwrap();
    dbg!(magic_non_consume(&mut buf, b"IMPM"));
    // buf.read_u16_le().unwrap();

    // let flag = buf.read_byte().unwrap();
    // dbg!(String::from_utf8_lossy(b));
    let flags: u8 = 0b_0000_1000;
    dbg!(flags.is_set(flag::STEREO));

    // flag::STEREO.is_set(flag);
    // dbg!(magic_non_consume(&mut buf, b"Hello"));
    // dbg!(buf.read_u16_le());
    // dbg!(buf.read_u16_le());
    // Ripper::new("jsljfkla");
    // Ripper::from_buf();
    // Ripper::pcm()
}

/// Experimental
fn validate<R: ByteReader>(buf: &mut R) -> io::Result<()> {
    bad_magic_non_consume(buf, b"ziRCON")?;
    magic_non_consume(buf, b"IMPM")?;
    buf.skip_bytes(4)?;
    let title = buf.read_bytes_boxed_slice(20)?;
    let ord_num = buf.read_u16_le()?;
    let ins_num = buf.read_u16_le()?;
    let smp_num = buf.read_u16_le()?;
    let compat_ver = buf.read_u16_le()?;
    let skip_offset = 0x00c0 + ord_num + (ins_num * 4);
    buf.set_seek_pos(skip_offset as u64)?;
    let mut smp_ptrs: Vec<u32> = Vec::with_capacity(smp_num as usize);
    
    for _ in 0..smp_num {
        smp_ptrs.push(buf.read_u32_le()?);
    }
    
    Ok(())
}

// fn samples_filtered<'b>(smp: &'b [Sample]) -> Vec<&'b Sample> {
//     smp.iter().filter(|smp| smp.len != 0).collect()
// }

fn build_samples<R: ByteReader>(reader: &mut R, ptrs: Vec<u32>) -> Vec<Sample>{
    let mut sample_data: Vec<Sample> = Vec::with_capacity(ptrs.len());
    // reader.size();
    todo!()
}
