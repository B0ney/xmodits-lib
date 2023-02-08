use crate::interface::Error;
use crate::parser::bytes::le_u16 as _le_u16;
use bytemuck::cast_slice;
use log::warn;

fn le_u16(buf: &[u8], offset: usize) -> Result<u16, Error> {
    _le_u16(buf, offset).ok_or_else(Error::bad_sample)
}

fn get_byte(buf: &[u8], offset: usize) -> Result<u8, Error> {
    buf.get(offset).ok_or_else(Error::bad_sample).copied()
}

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
#[rustfmt::skip] 
pub fn decompress_8_bit(buf: &[u8], mut len: u32, it215: bool) -> Result<Vec<u8>, Error> {
    let mut blklen: u16;                // uncompressed block length. Usually 0x8000 for 8-Bit samples
    let mut blkpos: u16;                // block position
    let mut sample_value: i8;           // decompressed sample value             (Note i8 for 8 bit samples)
    let mut d1: i8;                     // integrator buffer for IT2.14          (Note i8 for 8 bit samples)
    let mut d2: i8;                     // second integrator buffer for IT2.15   (Note i8 for 8 bit samples)
    let mut width: u8;                  // Bit width. (Starts at 9 For 8-Bit samples)
    let mut value: u16;                 // Value read (Note u16 for 8-bit samples)
    let mut dest_buf: Vec<u8>           = Vec::with_capacity(len as usize); // Buffer to write decompressed data
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
                warn!("Failed to fully extract this sample due to an invalid bit width: {}. (Should be < 10)", width);
                return Ok(dest_buf);
                // return Err(Error::Extraction(format!("Invalid Bit width. Why is it {}?", width)));
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
    Ok(dest_buf)
}

#[inline(always)]
#[rustfmt::skip]
pub fn decompress_16_bit(buf: &[u8], mut len: u32, it215: bool) -> Result<Vec<u8>, Error> {
    let mut blklen: u16;                // uncompressed block length. Usually 0x4000 for 16-Bit samples
    let mut blkpos: u16;                // block position
    let mut sample_value: i16;          // decompressed sample value             (Note i16 for 16 bit samples)
    let mut d1: i16;                    // integrator buffer for IT2.14          (Note i16 for 16 bit samples)
    let mut d2: i16;                    // second integrator buffer for IT2.15   (Note i16 for 16 bit samples)
    let mut width: u8;                  // Bit width. (Starts at 17 For 16-Bit samples)
    let mut value: u32;                 // Value read (Note u32 for 16 bit sample)
    let mut dest_buf: Vec<u8>           = Vec::with_capacity(len as usize); // Buffer to write decompressed data
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
                warn!("Failed to fully extract this sample due to an invalid bit width: {}. (Should be < 18)", width);
                return Ok(dest_buf);
                // return Err(Error::Extraction(format!("Invalid Bit width. Why is it {}?", width)));
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

    Ok(dest_buf)
}
