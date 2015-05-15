#![allow(dead_code)]
extern crate wavout;
extern crate rand;

use wavout::F32Sample;

mod synth;
use synth::ksstring::KarplusStrong;

fn main() {

    //let y = generate_tone(2.0f64, 440.0f64, 44100, 32);
    //let mut x = create_mono_wave_file(y, 44100, 32);

    //use std::fs::File;
    //use std::io::prelude::*;
    //use std::io::copy;
    //let mut f = File::create("test_440.wav").unwrap();
    //let _ = copy( &mut x.header, &mut f);
    //let _ = copy( &mut x.format_chunk, &mut f);
    //let _ = copy( &mut x.data, &mut f);
    //f.sync_all();*/
}
