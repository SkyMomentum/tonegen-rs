use rand;

pub struct KarplusStrong {
    ring: Vec<f32>,
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

    pub fn with_frequency(freq: f64) -> KarplusStrong {
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

    pub fn set_frequency(&mut self, freq: f64) {
        self.frequency = freq;
        //if freq requires a longer Vec, reallocate
        self.ring_size = (self.sample_rate / self.frequency).round() as usize;
        let cap = self.ring.capacity();
        if cap < self.ring_size {
            self.ring.reserve((self.ring_size - cap));
        }
    }

    pub fn pluck(&mut self) {
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
    
    pub fn sample(&mut self) -> f32 {
        *self.ring.get(self.ring_first).unwrap()
    }

    pub fn tick_simulation(&mut self) {
        let next_index = self.ring_first + 1;
        let mut second: f32;
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

    pub fn get_ticks(&mut self) -> u64 {
        self.ticks
    }

    pub fn get_ring_size(&self) -> usize {
        self.ring_size
    }
}

