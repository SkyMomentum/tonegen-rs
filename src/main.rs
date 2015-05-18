//[allow(dead_code)]
extern crate wavfile;
extern crate rand;

use std::env;
use std::fs::File;
use std::io::copy;

use wavfile::{create_mono_wav, create_mono_datachunk};
//use wavfile::F32Sample;

mod synth;
use synth::ksstring::generate_one_pluck_sample;
use synth::tone::generate_tone_f32;

mod options;

fn main() {
    let sample_rate = 44100.0;
    let opts = options::setup_options();
    let args: Vec<String> = env::args().collect();

    let dir_sep = if cfg!(target_family = "windows") {
        "\\"
    } else {
        "/"
    };
    
    let arg_zero = env::args().nth(0).unwrap();
    let exec_name = arg_zero.split(dir_sep).last().unwrap();
    
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

    let runtime: f64 = matches.opt_str("length").expect("Error: length parameter")
                              .parse().ok().expect("Error: length parameter");
    let freq: f64 = matches.opt_str("frequency").expect("Error: frequency parameter")
                           .parse().ok().expect("Error: frequency parameter");
    let filename: String = matches.opt_str("out-file").expect("Error: Filename parameter");

    if (runtime > 0.0) && (freq > 0.0) {
        let dc = if matches.opt_present("k") {
            let kspluck = generate_one_pluck_sample(runtime, freq, sample_rate);
            create_mono_datachunk(kspluck)
        } else {
            let tone = generate_tone_f32(runtime, freq, sample_rate);
            create_mono_datachunk(tone)
        };
        
        let mut wav = create_mono_wav(dc, 44100, 32);

        let mut f = File::create(filename).unwrap();
        let _ = copy( &mut wav.header, &mut f);
        let _ = copy( &mut wav.format_chunk, &mut f);
        let _ = copy( &mut wav.data, &mut f);
        let _ = f.sync_all();
    } else {
        println!("Please enter sane values for parameters.")
    }

}
