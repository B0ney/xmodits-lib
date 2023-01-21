use std::{io::{self, Read, Seek, Cursor, SeekFrom, BufReader}, vec};
// use std
trait Data: Read + Seek + Send + Sync {
    fn size(&self) -> Option<u64>;
}

impl <T: AsRef<[u8]> + Send + Sync> Data for Cursor<T> {
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

impl <T: Data> Data for BufReader<T> {
    fn size(&self) -> Option<u64> {
        // self.stream_len()   
        None
    }
}

impl <T: Data>ByteReader for T {
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
}

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

trait ByteReader {
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
    fn read_bytes_boxed_slice(&mut self, bytes: usize) -> io::Result<Box<[u8]>>;
}

#[test]
fn g() {
    let mut buf = Cursor::new(vec![1,2,3,4,5,6]);
    DataStream::new(Box::new(buf));
    let mut buf = BufReader::new(std::fs::File::open("path").unwrap());
    let header = buf.read_bytes_boxed_slice(2).unwrap();
    
    // let buf = DataStream::new(Box::new(buf));
    dbg!(buf.read_u16_le());
    dbg!(buf.read_u16_le());
    Ripper::new("jsljfkla");
    Ripper::from_buf();
    Ripper::pcm()
    
}