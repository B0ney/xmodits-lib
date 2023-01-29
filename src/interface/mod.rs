pub mod error;
pub mod sample;
pub mod module;
pub mod audio;
// pub use sample;
// pub use Module;
// pub use error;
// pub use sample::{Sample, Depth, Channel};
use byteorder::ReadBytesExt;
pub use error::Error;

use std::{
    borrow::Cow,
    io::{Cursor, Read, Seek, Write},
    path::Path,
};

use crate::parser::to_str_os;

use self::sample::Depth;







#[inline]
fn make_signed(buf: &mut [u8], depth: Depth) {
    match depth {
        Depth::U16 => make_signed_16bit(buf),
        Depth::U8 => make_signed_8bit(buf),
        _ => unreachable!("Logic error"), // should be safe to ignore rather than panicking...
    }
}

#[inline]
fn make_signed_8bit(buf: &mut [u8]) {
    for i in buf {
        *i = i.wrapping_sub(i8::MAX as u8 + 1)
    }
}

#[inline]
fn make_signed_16bit(buf: &mut [u8]) {
    use byteorder::{ByteOrder, LE};

    for i in 0..(buf.len() / 2) {
        let idx: usize = i * 2;
        let new = LE::read_u16(&buf[idx..(idx + 2)]).wrapping_sub(i16::MAX as u16 + 1);
        LE::write_u16(&mut buf[idx..(idx + 2)], new);
    }
}


pub mod export;


#[test]
fn a() {
    // let mut A = Wav;
    let mut buf = vec![0];
    
    // A.write(Cow::Borrowed(b"Spam and eggs").as_ref(), &mut buf);
    dbg!(buf);
    // let mut file = Box::new(std::fs::File::create("path").unwrap());
    // let mut a = Dummy::load(vec![0]).unwrap();
    // a.pcm_into(8, &file).unwrap();
    // let mut c = IT::load(vec![0]).unwrap();
    // c.extract("h", |_,_| {todo!()})
    // let mut a= S3M;
    // we decouple the wav export
    // a.export::<WAV>("folder", nameer, )
}
