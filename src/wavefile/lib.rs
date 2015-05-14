use std::io::prelude::*;
use std::io::Result;
use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub struct WaveHeader {
	riff_header: [u8; 4], // "RIFF"
	file_size: u32,
	wave_header: [u8; 4], // "WAVE"
	read_cur: usize,
}

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

#[derive(Debug)]
pub struct DataChunk<T> {
	data_header: [u8; 4], // "data"
	size_data: u32,
	sample_vector: Vec<T>,
	read_cur: usize,
}

#[derive(Debug)]
pub struct WaveFile<T> {
	pub header: WaveHeader,
	pub format_chunk: FormatChunk,
	pub data: DataChunk<T>,
}

pub type F32Sample = f32;

//-------------------------------------------------
// Transmutation functions

macro_rules! transmute_to_u8_from {
	($t:ty, $num_bytes:expr, $targ:expr, $outbuf: expr) => ({
			use std::ptr::copy_nonoverlapping;
			unsafe {
				let buf = std::mem::transmute::<_, [u8; $num_bytes]>($targ);
				copy_nonoverlapping( (&buf).as_ptr(), $outbuf.as_mut_ptr(), $num_bytes );
			}
		});
}

macro_rules! do_transmute {
	($fnc:ident, $input:expr, $to_buf:expr, $offset:expr, $size:expr) => ({
			let mut transmutebuf: [u8; $size] = [0; $size];
			$fnc($input, &mut transmutebuf);
			*$offset = *$offset + append_bytes( &transmutebuf, $to_buf, *$offset);
		});
}

fn u16_to_u8(target: u16, output: &mut [u8]) {
	transmute_to_u8_from!(u16, 2, target, output);
}

fn u32_to_u8(target: u32, output: &mut [u8]) {
	transmute_to_u8_from!(u32, 4, target, output);
}

fn f32_to_u8(target: f32, output: &mut [u8]) {
	transmute_to_u8_from!(f32, 4, target, output); 
}

fn append_bytes(bytes: &[u8], buf: &mut[u8], offset: usize) -> usize {
	if bytes.len() + offset <= buf.len() {
		for i in 0 .. bytes.len() {
			buf[i + offset] = bytes[i];
		}
	}
	bytes.len()
}

fn zero_u8_array(targ: &mut [u8]) {
	for i in 0 .. targ.len() {
		targ[i] = 0;
	}
}

//-------------------------------------------------
impl WaveHeader {
	pub fn set_size(&mut self, size: u32){
		self.file_size = size;
	}
}

impl Default for WaveHeader {
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
	pub fn set_number_channels(&mut self, chans: u16) {
		self.number_channels = chans;
	}
	pub fn set_sample_rate(&mut self, rate: u32) {
		self.samples_second = rate;
	}
	pub fn set_byte_rate(&mut self, bytes: u32) {
		self.bytes_second = bytes;
	}
	pub fn set_block_align(&mut self, align: u16 ) {
		self.block_alignment = align;
	}
	pub fn set_bits_sample(&mut self, bits: u16) {
		self.bits_sample = bits;
	}
}

impl Default for FormatChunk {
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
		
		if self.read_cur >= 12 { return Ok(0); }
		
		let mut offset: usize = 0;
		let mut tmb: [u8; 24] = [0; 24];
		
		if buf.len() >= 24 {
			zero_u8_array(buf);
			for i in 0 .. 4 {
				tmb[i] = self.fmt_header[i];
			}
			offset = 4;
			
			do_transmute!(u32_to_u8, self.size_wave_chunk, &mut tmb, &mut offset, 4); 
			do_transmute!(u16_to_u8, self.wave_type_format, &mut tmb, &mut offset, 2);
			do_transmute!(u16_to_u8, self.number_channels, &mut tmb, &mut offset, 2);
			do_transmute!(u32_to_u8, self.samples_second, &mut tmb, &mut offset, 4);
			do_transmute!(u32_to_u8, self.bytes_second, &mut tmb, &mut offset, 4);
			do_transmute!(u16_to_u8, self.block_alignment, &mut tmb, &mut offset, 2);
			do_transmute!(u16_to_u8, self.bits_sample, &mut tmb, &mut offset, 2);
			append_bytes(&tmb, buf, 0);
		}
		self.read_cur = offset;
		Ok(offset)
	}
}

impl Read for WaveHeader {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		
		if self.read_cur >= 12 { return Ok(0); }
		
		if buf.len() >= 12 {
			zero_u8_array(buf);
			let mut tmb: [u8; 13] = [0; 13];
			let mut off: usize = 4;
			
			for i in 0 .. 4 {
				let x = i + 8;
				tmb[i] = self.riff_header[i];
				tmb[x] = self.wave_header[i];
			}
			do_transmute!(u32_to_u8, self.file_size, &mut tmb, &mut off, 4); 
			append_bytes(&tmb, buf, 0);
			off=12;
			self.read_cur = off;
			Ok(off)
		} else {
			Err( Error::new(ErrorKind::Other, "Insufficent buffer availible.") )
		}
	}
}

impl Read for DataChunk<F32Sample> {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let out_size: usize = self.size_data as usize + 4;
		let mut tmb: [u8; 4] = [0; 4];
		let mut off: usize = 0;
		let mut x: usize = 0;
		
		if self.read_cur >= out_size { return Ok(0); }
		if buf.len() > 8 {
			if self.read_cur == 0 {
				zero_u8_array(buf);
				for i in 0 .. 4 {
					buf[i] = self.data_header[i];
				}
				off = 4;
				do_transmute!(u32_to_u8, self.size_data, &mut tmb, &mut x, 4);
				off = off + append_bytes(&tmb, buf, off);
			}
			
			let (_, work_slice) = self.sample_vector.split_at( self.read_cur / 4 );

			for fl in work_slice {
				println!("fl {}", fl);
				x = 0;
				do_transmute!(f32_to_u8, *fl, &mut tmb, &mut x, 4);
				if (off + 4) < buf.len() {
					off = off + append_bytes(&tmb, buf, off);
				} else {
					self.read_cur = self.read_cur + off;
					return Ok(off);
				}
			}
			self.read_cur = self.read_cur + off;
			Ok(off)
		} else {
			Err( Error::new(ErrorKind::Other, "Insufficent buffer availible.") )
		}
	}
}

impl DataChunk<F32Sample> {
	pub fn push_sample(&mut self, sample: F32Sample) {
		self.sample_vector.push(sample)
	}
	pub fn set_size(&mut self, size: u32) {
		self.size_data = size;
	}
	pub fn len(&self) -> usize {
		self.sample_vector.len()
	}
}

impl Default for DataChunk<F32Sample> {
	fn default() -> DataChunk<F32Sample> {
		DataChunk {
			data_header: [b'd', b'a', b't', b'a'],
			size_data: 0,
			sample_vector: Vec::new(),
			read_cur: 0,
		}
	}
}

impl WaveFile<F32Sample> {
	pub fn create_new( hdr: WaveHeader, fmt: FormatChunk, data_in: DataChunk<F32Sample>) -> WaveFile<F32Sample> {
		WaveFile { header: hdr, format_chunk: fmt, data: data_in, }
	}
}