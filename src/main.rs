#![allow(dead_code)]
extern crate wavfile;
extern crate rand;

use std::env;
use std::fs::File;
use std::io::copy;

use wavfile::{create_mono_wav, create_mono_datachunk};
//use wavfile::F32Sample;

mod synth;
//use synth::ksstring::KarplusStrong;
use synth::tone::generate_tone_f32;

mod options;

fn main() {


    let opts = options::setup_options();
    //let matches = options::get_matches(&opts).clone();
    let args: Vec<String> = env::args().collect();
    let exec_name = env::args().nth(0).unwrap();
    let matches_result = opts.parse(&args[1..]);
    let matches = match matches_result {
        Ok(m) => { m }
        Err(e) => {
                println!("Error: {}", e.to_string());
                options::print_help(&opts, &exec_name );
                return; 
            }
    };

    if matches.opt_present("h") {
        options::print_help(&opts, &exec_name);
        return;
    }

    let mut filename = String::new();

    let mut runtime: f64 =  matches.opt_str("length").expect("Error: length parameter")
                      .parse().ok().expect("Error: length parameter");
    let mut freq: f64 = matches.opt_str("frequency").expect("Error: frequency parameter")
                  .parse().ok().expect("Error: frequency parameter");
    filename = matches.opt_str("frequency").expect("Error: Filename parameter")
                      .parse().ok().expect("Error: Filename parameter");

    if (runtime > 0.0) && (freq > 0.0) {
        let tone = generate_tone_f32(runtime, freq, 44100);
        let dc = create_mono_datachunk(tone);
        let mut wav = create_mono_wav(dc, 44100, 32);

        let mut f = File::create(filename).unwrap();
        let _ = copy( &mut wav.header, &mut f);
        let _ = copy( &mut wav.format_chunk, &mut f);
        let _ = copy( &mut wav.data, &mut f);
        f.sync_all();
    }

}
