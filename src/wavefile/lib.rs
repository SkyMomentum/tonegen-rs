use std::io::prelude::*;
use std::io::Result;
use std::io::{Error, ErrorKind};

/// Struct representing the header of a .wav
/// 
/// Is not packed because of additional read_cur for Read trait impl. Could probably be
/// done anyway.
///
#[derive(Debug)]
pub struct WaveHeader {
    riff_header: [u8; 4], // "RIFF"
    file_size: u32,
    wave_header: [u8; 4], // "WAVE"
    read_cur: usize,
}

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

/// Struct representing an overall .wav file with a single data chunk.
///
/// Artifact of thinking about packing the component structs then using unsafe mem operations to
/// write directly to disk. Instead implemented Read on component types.
#[derive(Debug)]
pub struct WaveFile<T> {
    pub header: WaveHeader,
    pub format_chunk: FormatChunk,
    pub data: DataChunk<T>,
}

/// Alias for only currently supported sample type.
pub type F32Sample = f32;

/// Transmute and copy to u8 array.
///
/// # Unsafe
/// Forces $targ to be represented as an arry of u8. $num_bytes is assumed to be the number of
/// bytes $targ occupies. $num_bytes must also be the same size as the $outbuf used. First param
/// is just for readability at use site.
/// 
macro_rules! transmute_to_u8_from {
    ($t:ty, $num_bytes:expr, $targ:expr, $outbuf: expr) => ({
            use std::ptr::copy_nonoverlapping;
            unsafe {
                let buf = std::mem::transmute::<_, [u8; $num_bytes]>($targ);
                copy_nonoverlapping( (&buf).as_ptr(), $outbuf.as_mut_ptr(), $num_bytes );
            }
        });
}

/// Setup buffer for transmutation then call into macro.
///
/// The macro setups the boiler plate to call transmute_to_u8_from! macro. $fnc is a fn that exists
/// to call transmute macro with correct parameters for converting a specific type.
///
/// ```
/// fn u16_to_u8(target: u16, output: &mut [u8]) {
///    transmute_to_u8_from!(u16, 2, target, output);
/// }
/// ```
/// Naming the function as x_to_u8 increases readability at use site. Having the target byte size
/// as last param doesn't.
/// ```
/// do_transmute!(u16_to_u8, input, &mut buf, &mut buf_offset, 2);
/// ```
macro_rules! do_transmute {
    ($fnc:ident, $input:expr, $to_buf:expr, $offset:expr, $size:expr) => ({
            let mut transmutebuf: [u8; $size] = [0; $size];
            $fnc($input, &mut transmutebuf);
            *$offset = *$offset + append_bytes( &transmutebuf, $to_buf, *$offset);
        });
}

/// Helper function to convert u16 to a [u8; 2].
fn u16_to_u8(target: u16, output: &mut [u8]) {
    transmute_to_u8_from!(u16, 2, target, output);
}
/// Helper function to convert u32 to a [u8; 4].
fn u32_to_u8(target: u32, output: &mut [u8]) {
    transmute_to_u8_from!(u32, 4, target, output);
}
/// Helper function to convert f32 to a [u8; 4].
fn f32_to_u8(target: f32, output: &mut [u8]) {
    transmute_to_u8_from!(f32, 4, target, output); 
}

/// Function to bytewise copy from &[u8] to an &mut [u8], inserting at offset/
fn append_bytes(bytes: &[u8], buf: &mut [u8], offset: usize) -> usize {
    if bytes.len() + offset <= buf.len() {
        for i in 0 .. bytes.len() {
            buf[i + offset] = bytes[i];
        }
    }
    bytes.len()
}

/// Utility fn to zero out a given &mut [u8].
fn zero_u8_array(targ: &mut [u8]) {
    for i in 0 .. targ.len() {
        targ[i] = 0;
    }
}

//-------------------------------------------------
impl WaveHeader {
	/// Set the size DWORD of .wav header.
    pub fn set_size(&mut self, size: u32){
        self.file_size = size;
    }
}

impl Default for WaveHeader {
	/// Initialize a header, including RIFF and WAVE markers.
    fn default() -> WaveHeader {
        WaveHeader { 
            riff_header: [b'R', b'I', b'F', b'F'],
            file_size: 0,
            wave_header: [b'W', b'A', b'V', b'E'],
            read_cur: 0,
        }
    }
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

impl Read for WaveHeader {
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

impl WaveFile<F32Sample> {
    /// Helper to create a new WaveFile of F32Samples.
    pub fn create_new( hdr: WaveHeader, fmt: FormatChunk, data_in: DataChunk<F32Sample>) -> WaveFile<F32Sample> {
        WaveFile { header: hdr, format_chunk: fmt, data: data_in, }
    }
}