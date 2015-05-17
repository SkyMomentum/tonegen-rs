use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::io::Result;

use super::F32Sample;
//#[macro_use]
use util::{zero_u8_array, append_bytes};

/// Struct for a data chunk of a .wav
///
/// Is not packed for same reason as WaveHeader. Currently doesn't handle data longer than the
/// max value of a u32.
#[derive(Debug)]
pub struct DataChunk<T> {
    data_header: [u8; 4], // "data"
    size_data: u32,
    sample_vector: Vec<T>,
    read_cur: usize,
}

impl Read for DataChunk<F32Sample> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let out_size: usize = self.size_data as usize + 4;
        // Temporary buffer for transmuting F32Sample, and u32 header size component.
        let mut tmb: [u8; 4] = [0; 4];
        let mut off: usize = 0;
        // number of writen bytes in transmute macro, but not used here anymore due to an early
        // rewrite on learning that the size of the data chunk exceeds the &buf sent by copy() 
        // which is 64k on my machine. Param still used at other two macro use sites.
        let mut x: usize = 0;
        
        // Send EOF if we've sent everything.
        if self.read_cur >= out_size { return Ok(0); }
        
        // Sanity check, space at least for chunk header.
        if buf.len() > 8 {
            // if we're starting the copy()
            if self.read_cur == 0 {
                // zero out buf, copy in DATA and size of data that follows then increment off.
                zero_u8_array(buf);
                for i in 0 .. 4 {
                    buf[i] = self.data_header[i];
                }
                off = 4;
                do_transmute!(u32_to_u8, self.size_data, &mut tmb, &mut x, 4);
                off = off + append_bytes(&tmb, buf, off);
            }
            
            // Split the Vec<F32Sample> of data at the read_cur position. 
            let (_, work_slice) = self.sample_vector.split_at( self.read_cur / 4 );

            // Loop over that slice.
            for fl in work_slice {
                // Reset temp buffer write position. Then transmute f32, alias F32Sample. 
                x = 0;
                do_transmute!(f32_to_u8, *fl, &mut tmb, &mut x, 4);
                // Check buf position before write, if full return Ok(off) for the bytes written.
                // increment read_cur for next call from copy(). 
                if (off + 4) < buf.len() {
                    off = off + append_bytes(&tmb, buf, off);
                } else {
                    self.read_cur = self.read_cur + off;
                    return Ok(off);
                }
            }
            // Upon reaching end of work slice without filling output buf update read_cur so EOF
            // is sent next pass. Return Ok(off) for bytes written on this pass.
            self.read_cur = self.read_cur + off;
            Ok(off)
        } else {
            // buffer not big enough for header even.
            Err( Error::new(ErrorKind::Other, "Insufficent buffer availible.") )
        }
    }
}

impl DataChunk<F32Sample> {
    /// Interface to push on member sample_vector.
    pub fn push_sample(&mut self, sample: F32Sample) {
        self.sample_vector.push(sample)
    }
    /// Set the size of the chunk.
    pub fn set_size(&mut self, size: u32) {
        self.size_data = size;
    }
    /// Interface to len on member sample_vector.
    pub fn len(&self) -> usize {
        self.sample_vector.len()
    }
}

impl Default for DataChunk<F32Sample> {
    /// Defaults for DataChunk of F32Sample.
    ///
    /// Include DATA marker and a new Vec<F32Samples>.
    fn default() -> DataChunk<F32Sample> {
        DataChunk {
            data_header: [b'd', b'a', b't', b'a'],
            size_data: 0,
            sample_vector: Vec::new(),
            read_cur: 0,
        }
    }
}

pub fn create_mono_datachunk(data: Vec<F32Sample>) -> DataChunk<F32Sample>{
    let mut dc: DataChunk<F32Sample> = Default::default();
    
    for x in data.iter() {
        dc.push_sample(*x);
    }
    let mut len: u32;
    {
        len = dc.len() as u32;
        len = len * 4;
    }
    dc.set_size(len);
    dc
}

pub fn create_stereo_datachunk(one: Vec<F32Sample>, two: Vec<F32Sample>) -> DataChunk<F32Sample> {
    use std::iter::Iterator;
    
    let mut dc: DataChunk<F32Sample> = Default::default();
    
    let li = one.iter();
    let ri = two.iter();
    let stereo = li.zip(ri);
    for x in stereo {
        let (l,r) = x;
        dc.push_sample(*l);
        dc.push_sample(*r);
    }
    let mut len: u32;
    {
        len = dc.len() as u32;
        len = len * 8;
    }
    dc.set_size(len);
    dc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_mono_datachunk_test_01() {
        // This probably does more than test create_mono... oh well.
        use std::io::copy;
        use std::io::Read;
        use super::super::util;

        // Setup a test payload, then convert it to a u8 array to compare against.
        let payload = [0.0f32, 0.1f32, 0.2f32, 0.3f32, 0.4f32, 0.5f32];
        let mut expected: [u8; 32] = [0;32];
        let mut x: usize = 4;
        expected[0] = b'D'; expected[1] = b'A'; expected[2] = b'T'; expected[3] = b'A';
        do_transmute!(u32_to_u8, 24, &mut expected, &mut x, 4);
        for j in payload.iter() {
            do_transmute!(f32_to_u8, *j, &mut expected, &mut x, 4);
        }

        // Setup audio sample vector.
        let mut dat: Vec<f32> = Vec::new();
        for x in payload.iter() {
            dat.push(*x);
        }

        // Finally call tested function then compare to expected.
        let mut rbuf: [u8; 32] = [0; 32];
        let mut retv = create_mono_datachunk(dat);
        retv.read(&mut rbuf);
        for i in 0 .. payload.len() {
            assert_eq!(rbuf[i], expected[i]);
        }
    }
}
