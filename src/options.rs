extern crate getopts;

use self::getopts::Options;

pub fn setup_options() -> Options {
    let mut opts = Options::new();
    
    opts.reqopt("f", "frequency", "Frequency of generated tone", "FREQ")
        .reqopt("l", "length", "Run length of generated wav.", "SECS")
        .reqopt("o", "out-file", "File name to write the wav file to", "FILE")
        .optflag("t", "tone", "Generate sine tone, default.")
        .optflag("k", "karplus-strong", "Generate a karplus strong sample from single pluck.")
        .optflag("s", "stereo", "Make a stereo .wav file")
        .optflag("h", "help", "Print this help.")
        .optflagopt("r", "repeat",
                    "Repeat the karplus-strong pluck when sample is below threshold", "THRESHOLD");
    opts
}

pub fn print_help(opts: &Options, name: &str) {
    let brief = format!("USE: {} [options]", name);
    print!("{}", opts.usage(&brief));
}

/*pub fn check_required_args(matches: &Matches) -> bool {
    matches.opt_present("f") && matches.opt_present("l") && matches.opt_present("o")
}*/