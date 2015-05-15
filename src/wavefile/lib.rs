use std::io::prelude::*;
use std::io::Result;
use std::io::{Error, ErrorKind};

#[macro_use] mod util;

mod formatchunk;
pub use formatchunk::FormatChunk;

mod wavheader;
pub use wavheader::WavHeader;

mod datachunk;
pub use datachunk::DataChunk;


/// Struct representing an overall .wav file with a single data chunk.
///
/// Artifact of thinking about packing the component structs then using unsafe mem operations to
/// write directly to disk. Instead implemented Read on component types.
#[derive(Debug)]
pub struct WaveFile<T> {
    pub header: WavHeader,
    pub format_chunk: FormatChunk,
    pub data: DataChunk<T>,
}

/// Alias for only currently supported sample type.
pub type F32Sample = f32;

//-------------------------------------------------

impl WaveFile<F32Sample> {
    /// Helper to create a new WaveFile of F32Samples.
    pub fn create_new( hdr: WavHeader, fmt: FormatChunk, data_in: DataChunk<F32Sample>) -> WaveFile<F32Sample> {
        WaveFile { header: hdr, format_chunk: fmt, data: data_in, }
    }
}

/// Function to package provided Datachunk as a mono .wav struct.
///
/// Current support functions only provide 32bit sample size.
fn create_mono_wave_file(data_in: DataChunk<F32Sample>, sample_rate: u32, sample_bits: u32) -> WaveFile<F32Sample> {

    let format_chunk_size = 24u32;
    let data_chunk_header_size = 8u32;

    let mut fmt: FormatChunk = Default::default();
    let num_channels = 1u32;
    fmt.set_sample_rate( sample_rate );
    fmt.set_byte_rate( num_channels * sample_rate * (sample_bits/8) );
    fmt.set_block_align( (num_channels * (sample_bits/8)) as u16 );
    fmt.set_bits_sample( sample_bits as u16 );

    let data_size: u32 = (data_in.len() as u32) * (sample_bits / 8);
    let total_size: u32 = data_size + format_chunk_size + data_chunk_header_size;

    let mut hdr: WavHeader = Default::default();
    hdr.set_size(total_size);

    let wave: WaveFile<F32Sample> = WaveFile::create_new(hdr,fmt,data_in);
    wave
}

fn create_mono_datachunk(data: Vec<F32Sample>) -> DataChunk<F32Sample>{
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

fn create_stereo_datachunk(one: Vec<F32Sample>, two: Vec<F32Sample>) -> DataChunk<F32Sample> {
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