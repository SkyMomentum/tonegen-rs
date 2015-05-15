
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
        // number of writed bytes in transmute macro, but not used here anymore due to an early
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
            data_header: [b'D', b'A', b'T', b'A'],
            size_data: 0,
            sample_vector: Vec::new(),
            read_cur: 0,
        }
    }
}
