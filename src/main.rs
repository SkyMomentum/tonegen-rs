use std::fs::File;
use std::io::prelude::*;

extern crate wavout;
extern crate rand;

use wavout::{F32Sample, DataChunk, FormatChunk, WaveHeader, WaveFile};

mod synth;


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
