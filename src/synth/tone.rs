use std::f64::consts;

/// Fill a Vec<> with a single cycle at frequency and sample rate.
pub fn create_sine_sample(frequency: f64, sample_rate: f64) -> Vec<f32> {

    let samples_sine: f64 = sample_rate / frequency;
    let samples_num: u32 = samples_sine.floor() as u32;
    let half_wave: f64 = samples_sine / 2.0f64;
    
    let mut tone_cycle: Vec<f32> = Vec::with_capacity(samples_num as usize);
    
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
pub fn generate_tone_f32(run_length: f64, frequency: f64, sample_rate: f64) -> Vec<f32> {

    let mut tone_out: Vec<f32> = Vec::new();

    let tone_cycle: Vec<f32> = create_sine_sample(frequency, sample_rate);
    let cycle_len: usize = tone_cycle.len();

    let sr: f64 = sample_rate as f64;
    let total_samples: u32 = ((run_length * sr).floor()) as u32;

    let mut out_counter: u32 = 0;
    let mut cycle_index: usize = 0;

    loop {
        if out_counter > total_samples { break; }
        if cycle_index >= cycle_len { cycle_index = 0; }
        tone_out.push( *(tone_cycle.get(cycle_index).unwrap()) );
        out_counter = out_counter + 1;
        cycle_index = cycle_index + 1;
    }

    tone_out
}
