use std::io::prelude::*;
use std::io::Result;

//#[macro_use]
use util::{zero_u8_array, append_bytes};

/// Struct for the format chunk of a .wav
///
/// Is not packed for same reason as WaveHeader.
#[derive(Debug)]
pub struct FormatChunk {
    fmt_header: [u8; 4],  // "fmt "
    size_wave_chunk: u32, // 16 - number of bytes blow
    wave_type_format: u16,// 1 - Linear PCM
    number_channels: u16,
    samples_second: u32,
    bytes_second: u32,
    block_alignment: u16,
    bits_sample: u16,
    read_cur: usize,
}

impl FormatChunk {
    /// Set number of channels for file. Default of 1.
    pub fn set_number_channels(&mut self, chans: u16) {
        self.number_channels = chans;
    }
    /// Set number of samples per second for data of file.
    pub fn set_sample_rate(&mut self, rate: u32) {
        self.samples_second = rate;
    }
    /// Set byte rate of file.
    /// ```text
    /// number of channels * sample byte site * sample rate
    /// ```
    pub fn set_byte_rate(&mut self, bytes: u32) {
        self.bytes_second = bytes;
    }
    /// Set block alignment of data in file.
    /// ```text
    /// number of channels * sample byte size
    /// ```
    pub fn set_block_align(&mut self, align: u16 ) {
        self.block_alignment = align;
    }
    /// Set number of bits per sample. Current library only supports 32 bit.
    pub fn set_bits_sample(&mut self, bits: u16) {
        self.bits_sample = bits;
    }
}

impl Default for FormatChunk {
    /// Default values for FormatChunk.
    ///
    /// Including "fmt " marker and chunk size. Also 1 channel and 32 bit sample size. 
    fn default() -> FormatChunk {
        FormatChunk {
            fmt_header: [b'f', b'm', b't', b' '],
            size_wave_chunk: 16,
            wave_type_format: 1,
            number_channels: 1,
            samples_second: 0,
            bytes_second: 0,
            block_alignment: 0,
            bits_sample: 32, //defaulting to F32Sample
            read_cur: 0,
        }
    }
}

impl Read for FormatChunk {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        
        // Send EOF if we've sent the header.
        if self.read_cur >= 24 { return Ok(0); }
        
        // Set up a temp buffer for transmutations.
        let mut offset: usize = 0;
        let mut tmb: [u8; 24] = [0; 24];
        
        // Sanity check then zero out buf, insert "FMT " marker.
        if buf.len() >= 24 {
            zero_u8_array(buf);
            for i in 0 .. 4 {
                tmb[i] = self.fmt_header[i];
            }
            offset = 4;
            
            // Transmute the rest of the chunk data into the temp buffer, offset
            // incrementing along.
            do_transmute!(u32_to_u8, self.size_wave_chunk, &mut tmb, &mut offset, 4); 
            do_transmute!(u16_to_u8, self.wave_type_format, &mut tmb, &mut offset, 2);
            do_transmute!(u16_to_u8, self.number_channels, &mut tmb, &mut offset, 2);
            do_transmute!(u32_to_u8, self.samples_second, &mut tmb, &mut offset, 4);
            do_transmute!(u32_to_u8, self.bytes_second, &mut tmb, &mut offset, 4);
            do_transmute!(u16_to_u8, self.block_alignment, &mut tmb, &mut offset, 2);
            do_transmute!(u16_to_u8, self.bits_sample, &mut tmb, &mut offset, 2);
            // Copy the temp buf to the output buf. Was easier than direct insert by the
            // macro, for now.
            append_bytes(&tmb, buf, 0);
        }
        // Update cursor then send Ok()
        self.read_cur = offset;
        Ok(offset)
    }
}