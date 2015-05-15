use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::io::Result;

#[macro_use]
use util;

use util::{zero_u8_array, append_bytes};

/// Struct representing the header of a .wav
/// 
/// Is not packed because of additional read_cur for Read trait impl. Could probably be
/// done anyway.
///
#[derive(Debug)]
pub struct WavHeader {
    riff_header: [u8; 4], // "RIFF"
    file_size: u32,
    wave_header: [u8; 4], // "WAVE"
    read_cur: usize,
}

impl WavHeader {
    /// Set the size DWORD of .wav header.
    pub fn set_size(&mut self, size: u32){
        self.file_size = size;
    }
}

impl Default for WavHeader {
    /// Initialize a header, including RIFF and WAVE markers.
    fn default() -> WavHeader {
        WavHeader { 
            riff_header: [b'R', b'I', b'F', b'F'],
            file_size: 0,
            wave_header: [b'W', b'A', b'V', b'E'],
            read_cur: 0,
        }
    }
}

impl Read for WavHeader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // If we've passed everything on already then return EOF by returning Ok(0).
        if self.read_cur >= 12 { return Ok(0); }
        
        if buf.len() >= 12 {
            // Clean buffer then set up a temp buffer for the header
            zero_u8_array(buf);
            let mut tmb: [u8; 13] = [0; 13];
            // offset for inset of filesize dword
            let mut off: usize = 4;
            
            // Copy the constant RIFF and WAVE bytewise  
            for i in 0 .. 4 {
                let x = i + 8;
                tmb[i] = self.riff_header[i];
                tmb[x] = self.wave_header[i];
            }
            // insert the u32 representing file size after RIFF. and copy it to the temp buf
            do_transmute!(u32_to_u8, self.file_size, &mut tmb, &mut off, 4); 
            append_bytes(&tmb, buf, 0);
            // offset set to account for WAVE inserted in loop.
            off=12;
            // update position for nextime copy() calls.
            self.read_cur = off;
            // return bytes written
            Ok(off)
        } else {
            Err( Error::new(ErrorKind::Other, "Insufficent buffer availible.") )
        }
    }
}
