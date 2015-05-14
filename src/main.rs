use std::f64::consts;
use std::fs::File;
use std::io::prelude::*;

extern crate wavout;
extern crate rand;

use wavout::{F32Sample, DataChunk, FormatChunk, WaveHeader, WaveFile};

/// Fill a Vec<> with a single cycle at frequency and sample rate.
fn create_sine_sample(frequency: f64, sample_rate: u32) -> Vec<F32Sample> {

    let sr: f64 = sample_rate as f64;
    let samples_sine: f64 = sr / frequency;
    let samples_num: u32 = samples_sine.floor() as u32;
    let half_wave: f64 = samples_sine / 2.0f64;
    
    let mut tone_cycle: Vec<F32Sample> = Vec::with_capacity(samples_num as usize);
    
    for i in 0..samples_num {
        let x: f64 = i as f64;
        let val_calc: f64 = ((x * consts::PI)/half_wave).sin();
        let val_out: f32 = val_calc as f32;
        tone_cycle.push(val_out);
    }

    tone_cycle
}

/// Create a data chunk with specified run length, frequency, and sample details.
///
/// Current support functions only provide 32bit sample size.
fn generate_tone(run_length: f64, frequency: f64, sample_rate: u32, sample_bits: u32) -> DataChunk<F32Sample> {

    let mut tone_out: DataChunk<F32Sample> = Default::default();

    let tone_cycle: Vec<F32Sample> = create_sine_sample(frequency, sample_rate);
    let cycle_len: usize = tone_cycle.len();

    let sr: f64 = sample_rate as f64;
    let total_samples_f64: f64 = (run_length * sr).floor();
    let total_samples: u32 =  total_samples_f64 as u32;

    let mut out_counter: u32 = 0;
    let mut cycle_index: usize = 0;

    loop {
        if out_counter > total_samples { break; }
        if cycle_index >= cycle_len { cycle_index = 0; }
        tone_out.push_sample( *(tone_cycle.get(cycle_index).unwrap()) );
        out_counter = out_counter + 1;
        cycle_index = cycle_index + 1;
    }

    tone_out.set_size( out_counter * (sample_bits / 8) );
    tone_out
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

    let mut hdr: WaveHeader = Default::default();
    hdr.set_size(total_size);

    let wave: WaveFile<F32Sample> = WaveFile::create_new(hdr,fmt,data_in);
    wave
}

struct KarplusStrong {
    ring: Vec<F32Sample>,
    ring_first: usize,
    ring_last: usize,
    ring_size: usize,
    
    sample_rate: f64,
    frequency: f64,
    ticks: u64,
}

impl KarplusStrong {
    fn with_ring_size(size: usize) -> KarplusStrong {
        KarplusStrong {
            ring: Vec::with_capacity(size),
            ring_first: 0,
            ring_last: size,
            ring_size: size,
            sample_rate: 44100.0f64,
            frequency: 0.0f64,
            ticks: 0,
        }
    }

    fn with_frequency(freq: f64) -> KarplusStrong {
        let sample_rate = 44100.0f64;
        let size = (sample_rate / freq).round() as usize;
        let mut ks = KarplusStrong {
            ring: Vec::new(),
            ring_first: 0,
            ring_last: size,
            ring_size: size,
            sample_rate: 44100.0f64,
            frequency: freq,
            ticks: 0,
        };

        let mut i = 0 ;
        loop {
            ks.ring.push( 0.0f32 );
            i = i + 1;
            if i > size { break; }
        }
        ks
    }

    fn set_frequency(&mut self, freq: f64) {
        self.frequency = freq;
        //if freq requires a longer Vec, reallocate
        self.ring_size = (self.sample_rate / self.frequency).round() as usize;
        let cap = self.ring.capacity();
        if cap < self.ring_size {
            self.ring.reserve((self.ring_size - cap));
        }
    }

    fn pluck(&mut self) {
        use rand::distributions::{IndependentSample, Range};
        // Fill self.ring with random noise in range -0.5 to +0.5
        let mut random: rand::OsRng = rand::OsRng::new().unwrap();
        //let rng: &rand
        let between = Range::new(-0.50f32, 0.50f32);
        let mut i: usize = 0;

        loop {
            let y_val = between.ind_sample(&mut random);
            let samp = self.ring.get_mut(i).unwrap();
            *samp = y_val;
            i = i + 1;
            if i > self.ring_size { break; }
        }

    }
    
    fn sample(&mut self) -> F32Sample {
        *self.ring.get(self.ring_first).unwrap()
    }

    fn tick_simulation(&mut self) {
        let next_index = self.ring_first + 1;
        let mut second: f32 = 0.0f32;
        // Block to allow borrowing ring twice.
        {
            let sr: &f32 = self.ring.get(next_index).unwrap();
            second = *sr;
        }
        let first: &mut f32 = self.ring.get_mut(self.ring_first).unwrap();
        // Karplus-Strong
        *first = (*first + second) * 0.50f32 * 0.9940f32;
        self.ring_last = self.ring_first;
        if next_index >= self.ring_size { self.ring_first = 0; }
        else { self.ring_first = next_index; }
        self.ticks = self.ticks + 1;
    }

    fn get_ticks(&mut self) -> u64 {
        self.ticks
    }

    fn get_ring_size(&self) -> usize {
    	self.ring_size
    }
}



fn main() {
    use std::io::copy;

    let mut ks: KarplusStrong = KarplusStrong::with_frequency(440.0f64);
    
    ks.pluck();
    
    let mut i = 0;
    loop {
        ks.tick_simulation();
        let t = ks.get_ticks();
        let samp: F32Sample = ks.sample();
        println!("t={} s={}", t, samp);
        if i > 1000 { break; }
        i = i + 1;
    }
    //let y = generate_tone(2.0f64, 440.0f64, 44100, 32);
    //let mut x = create_mono_wave_file(y, 44100, 32);
    //let mut z: Vec<u8> = Vec::new();

    /*let _ = x.header.read_to_end(&mut z);
    println!("header - {:?} ", z);
    z.clear();
    let _ = x.format_chunk.read_to_end(&mut z);
    println!("fmt - {:?} ", z);*/

    /*let mut f = File::create("test_440.wav").unwrap();

    //try!( f.write_all( x.header.read_to_end()));
    let _ = copy( &mut x.header, &mut f);
    //try!( f.write_all( x.format_chunk.read_to_end()));
    let _ = copy( &mut x.format_chunk, &mut f);
    //try!( f.write_all( x.data.read_to_end()));
    let _ = copy( &mut x.data, &mut f);
    f.sync_all();*/

    
}
