// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Rust implementation for decompressing compressed Impulse Tracker samples
//!
//! Algorithm for sample decompression:
//!     https://github.com/nicolasgramlich/AndEngineMODPlayerExtension/blob/master/jni/loaders/itsex.c
//!
//! Bitreading:
//!     https://github.com/Konstanty/libmodplug/blob/master/src/load_it.cpp#L1183

use crate::error;
use crate::interface::Error;
use crate::parser::bytes::le_u16 as _le_u16;
use bytemuck::cast_slice;

#[rustfmt::skip] 
struct BitReader<'a> {
    block_offset: usize,    // Location of next block
    bitnum: u8,             // Bits left. When it hits 0, it resets to 8 & "blk_index" increments by 1.
    bitbuf: u32,            // Internal buffer for storing read bits
    buf: &'a [u8],          // IT Module buffer (read-only because reading data shouldn't modify anything)
    blk_index: usize,       // Used to index blk_data.
}

impl<'a> BitReader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self {
            bitnum: 0,
            bitbuf: 0,
            buf,
            blk_index: 0,
            block_offset: 0x0000,
        }
    }

    fn read_next_block(&mut self) -> Result<(), Error> {
        // First 2 bytes combined to u16 (LE). Tells us size of compressed block.
        let block_size = le_u16(self.buf, self.block_offset)?;

        // Set to 2 to skip length field
        self.blk_index = 2 + self.block_offset;

        // Initialize bit buffers.
        // (Following the original by setting it to 0 caused a lot of headaches :D)
        self.bitbuf = get_byte(self.buf, self.blk_index)? as u32;
        self.bitnum = 8;
        self.block_offset += block_size as usize + 2;
        Ok(())
    }

    fn offset(&self) -> usize {
        self.block_offset
    }

    fn read_bits_u16(&mut self, n: u8) -> Result<u16, Error> {
        Ok(self.read_bits_u32(n)? as u16)
    }

    fn read_bits_u32(&mut self, n: u8) -> Result<u32, Error> {
        if n == 0 {
            return Ok(0);
        }
        let mut value: u32 = 0;

        for _ in 0..n {
            if self.bitnum == 0 {
                self.blk_index += 1;
                self.bitbuf = get_byte(self.buf, self.blk_index)? as u32;
                self.bitnum = 8;
            }
            value >>= 1;
            value |= self.bitbuf << 31;
            self.bitbuf >>= 1;
            self.bitnum -= 1;
        }

        Ok(value >> (32 - n))
    }
}

#[inline(always)]
pub fn decompress_8_bit(buf: &[u8], len: u32, it215: bool, stereo: bool) -> Result<Vec<u8>, Error> {
    let mut out = Vec::with_capacity(len as usize);

    if stereo {
        let offset = decompress_8_bit_inner(buf, len*2, it215, &mut out)?;
        decompress_8_bit_inner(&buf[offset..], len*2, it215, &mut out)?;
    } else {
        decompress_8_bit_inner(buf, len, it215, &mut out)?;
    };

    return Ok(out)
}

#[rustfmt::skip] 
pub fn decompress_8_bit_inner(buf: &[u8], mut len: u32, it215: bool, dest_buf: &mut Vec<u8>) -> Result<usize, Error> {
    let mut blklen: u16;                // uncompressed block length. Usually 0x8000 for 8-Bit samples
    let mut blkpos: u16;                // block position
    let mut sample_value: i8;           // decompressed sample value             (Note i8 for 8 bit samples)
    let mut d1: i8;                     // integrator buffer for IT2.14          (Note i8 for 8 bit samples)
    let mut d2: i8;                     // second integrator buffer for IT2.15   (Note i8 for 8 bit samples)
    let mut width: u8;                  // Bit width. (Starts at 9 For 8-Bit samples)
    let mut value: u16;                 // Value read (Note u16 for 8-bit samples)
    let mut bitreader: BitReader        = BitReader::new(buf);

    // Unpack data
    while len != 0 {
        // Read new block, reset variables
        bitreader.read_next_block()?;

        // Make sure block len won't exceed len.
        blklen = if len < 0x8000 { len as u16 } else { 0x8000 };
        blkpos = 0;
        width = 9;
        d1 = 0; 
        d2 = 0;

        while blkpos < blklen {

            if width > 9 {
                error!("Could not fully decompress this sample because it has an invalid bit width: {}. (Should be < 10)", width);
                return Ok(bitreader.offset());
                // return Err(Error::Extraction(format!("Could not decompress this Impulse Tracker sample because it has an invalid bit width '{}' (Should be < 10)", width)));
            };

            value = bitreader.read_bits_u16(width)?;
        
            if width < 7 { // Method 1, 1-6 bits

                if value == (1 << (width - 1)) as u16
                {
                    value = bitreader.read_bits_u16(3)? + 1;

                    let val = value as u8;
                    width = if val < width { val } else { val + 1 };
                    continue;
                }
            
            } else if width < 9 { // Method 2, 7-8 bits
                let border: u16 = (0xff >> (9 - width)) - 4;

                if value > border
                    && value <= (border + 8)
                    {
                        value -= border;

                        let val = value as u8;
                        width = if val < width { val } else { val + 1 };
                        continue;
                    }

            } else {  // Method 3, 9 bits

                if (value & 0x100) >> 8 == 1 // is bit 8 set? 
                { 
                    width = ((value + 1) & 0xff) as u8;
                    continue;
                }
                
            }

            // sample values are encoded with "bit width"
            // expand them to be 8 bits
            // expand value to signed byte
            if width < 8 {
                let shift: u8 = 8 - width;
                sample_value = (value << shift) as i8 ;
                sample_value >>= shift as i8;
            } else {
                sample_value = value as i8;
            }

            // integrate
            // In the original C implementation, 
            // values will wrap implicitly if they overflow
            d1 = d1.wrapping_add(sample_value);
            d2 = d2.wrapping_add(d1);

            dest_buf.push(
                if it215 { d2 as u8 } else { d1 as u8 }
            );

            blkpos += 1;
        }

        len -= blklen as u32; 
    }
    Ok(bitreader.offset())
}

#[inline(always)]
pub fn decompress_16_bit(buf: &[u8], len: u32, it215: bool, stereo: bool) -> Result<Vec<u8>, Error> {
    let mut out = Vec::with_capacity(len as usize * 2);

    if stereo {
        let offset = decompress_16_bit_inner(buf, len*2, it215, &mut out)?;
        decompress_16_bit_inner(&buf[offset..], len*2, it215, &mut out)?;
    } else {
        decompress_16_bit_inner(buf, len, it215, &mut out)?;
    };

    return Ok(out)
}


#[rustfmt::skip]
pub fn decompress_16_bit_inner(buf: &[u8], len: u32, it215: bool, dest_buf: &mut Vec<u8>) -> Result<usize, Error> {
    let mut len = len / 2;         // Length of uncompressed sample. We half this we're decompressing a &[u16] as a &[u8]
    let mut blklen: u16;                // uncompressed block length. Usually 0x4000 for 16-Bit samples
    let mut blkpos: u16;                // block position
    let mut sample_value: i16;          // decompressed sample value             (Note i16 for 16 bit samples)
    let mut d1: i16;                    // integrator buffer for IT2.14          (Note i16 for 16 bit samples)
    let mut d2: i16;                    // second integrator buffer for IT2.15   (Note i16 for 16 bit samples)
    let mut width: u8;                  // Bit width. (Starts at 17 For 16-Bit samples)
    let mut value: u32;                 // Value read (Note u32 for 16 bit sample)
    let mut bitreader: BitReader        = BitReader::new(buf);

    while len != 0 {
        // Read new block, reset variables
        bitreader.read_next_block()?;

        // Make sure block len won't exceed len.
        blklen = if len < 0x4000 { len as u16 } else { 0x4000 };
        blkpos = 0;
        width = 17;
        d1 = 0; 
        d2 = 0;
        
        while blkpos < blklen {

            if width > 17 {
                error!("Could not fully decompress this sample because it has an invalid bit width: {}. (Should be < 18)", width);
                return Ok(bitreader.offset());
                // return Err(Error::Extraction(format!("Could not decompress this Impulse Tracker sample because it has an invalid bit width '{}' (Should be < 18)", width)));
            }

            value = bitreader.read_bits_u32(width)?;

            if width < 7 { // Method 1, 1-6 bits
                
                if value == (1 << (width - 1)) as u32
                {
                    value = bitreader.read_bits_u32(4)? + 1;

                    let val = value as u8;
                    width = if val < width { val } else { val + 1 };
                    continue;
                }
            
            } else if width < 17 { // Method 2, 7-16 bits
                let border: u32 = (0xffff >> (17 - width)) - 8;

                if value > border
                    && value <= (border + 16)
                    {
                        value -= border;

                        let val = value as u8;
                        width = if val < width { val } else { val + 1 };
                        continue;
                    }

            } else {  // Method 3, 17 bits
                if (value & 0x10000) >> 16 == 1 // is bit 16 set? 
                { 
                    width = ((value + 1) & 0xff) as u8;
                    continue;
                }
            }

            if width < 16 {
                let shift: u8 = 16 - width;
                sample_value = (value << shift) as i16 ;
                sample_value >>= shift as i16;
            } else {
                sample_value = value as i16;
            }

            d1 = d1.wrapping_add(sample_value);
            d2 = d2.wrapping_add(d1);

            let buf = if it215 { d2 } else { d1 };
            dest_buf.extend_from_slice(cast_slice(&[buf]));

            blkpos += 1;
        }

        len -= blklen as u32; 
    }

    Ok(bitreader.offset())
}

fn eof_err(len: usize, offset: usize) -> Error {
    let error = format!(
        "Unexpected EOF for compressed Impulse Tracker sample ({len} bytes) for given offset {offset}"
    );
    error!("{}", error);
    Error::Extraction(error)
}

fn le_u16(buf: &[u8], offset: usize) -> Result<u16, Error> {
    _le_u16(buf, offset).ok_or_else(|| eof_err(buf.len(), offset))
}

fn get_byte(buf: &[u8], offset: usize) -> Result<u8, Error> {
    buf.get(offset)
        .ok_or_else(|| eof_err(buf.len(), offset))
        .copied()
}

#[cfg(test)]
mod tests {
    use super::BitReader;
    use crate::interface::Error;

    #[test]
    fn readbit() -> Result<(), Error> {
        let buf: Vec<u8> = vec![
            0x1,
            0x0, // block size header (LE) of 1 byte
            0b1111_1110,
            0b1111_1111, // group 1
            0b1010_1110, // group 2
            0b1100_1100,
            0b1100_1111, // group 3
            0b0011_1010,
            0b1010_1010,
            0b1100_1100,
            0b1100_1100,
            0b1010_1010,
            0b1010_1010,
            0b1100_1100,
        ];
        let mut b = BitReader::new(&buf);
        b.read_next_block()?;
        // b.read_bits_u16(0)?;

        // test group 1
        assert_eq!(b.read_bits_u16(8)?, 0b_1111_1110);
        assert_eq!(b.read_bits_u16(8)?, 0b_1111_1111);

        // test group 2
        assert_eq!(b.read_bits_u16(4)?, 0b_0000_1110);
        assert_eq!(b.read_bits_u16(4)?, 0b_0000_1010);

        // test group 3
        assert_eq!(b.read_bits_u16(16)?, 0b_1100_1111_1100_1100);
        assert_eq!(b.read_bits_u16(9)?, 0b_0_0011_1010);

        Ok(())
    }
}
