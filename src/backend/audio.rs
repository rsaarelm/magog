use std::u16;
use num::traits::{Float};
use time;
use cpal;

pub struct Mixer {
    waves: Vec<Wave>,
    t: f64,
    rate: u32,
}

struct Wave {
    f: Box<Fn(f64) -> f64 + Send>,
    begin_t: f64,
    end_t: f64,
}

impl Mixer {
    pub fn new() -> Mixer {
        Mixer {
            waves: Vec::new(),
            t: time::precise_time_s(),
            rate: 44100,
        }
    }

    pub fn add_wave(&mut self, f: Box<Fn(f64) -> f64 + Send>, duration: f64) {
        assert!(duration >= 0.0);
        let t = time::precise_time_s();
        self.waves.push(Wave {
            f: f,
            begin_t: t,
            end_t: t + duration
        });
    }

    /// Runs the Audio mixer, takes over the thread.
    pub fn run(&mut self) {
        // TODO: Return a channel you can use to send new waves.
        self.t = time::precise_time_s();

        let mut voice = cpal::Voice::new();
        loop {
            {
                let mut buffer =
                    voice.append_data(1, cpal::SamplesRate(self.rate), 32768);

                for sample in buffer.iter_mut() {
                    *sample = self.tick();
                }
            }

            voice.play();
        }
    }

    fn tick(&mut self) -> u16 {
        let local_t = self.t;
        // Advance internal time by the length of the sample interval.
        self.t += 1.0 / self.rate as f64;

        // Remove expired waves.
        self.waves.retain(|w| w.end_t > local_t);

        let mut mixed = 0.0;

        for wave in self.waves.iter() {
            let wave_t = self.t - wave.begin_t;
            if wave_t < 0.0 { continue; }

            let a = (wave.f)(wave_t);
            assert!(-1.0 <= a && a <= 1.0);
            // Do some hacky equalization to keep summed waves from
            // going outside the wave envelope.
            mixed = a + mixed - a.signum() * 0.0.max(a * mixed);
            assert!(-1.0 <= mixed && mixed <= 1.0);
        }

        // FIXME: https://github.com/tomaka/cpal/issues/39
        // CPAL samples operate in the i16 space but must be cast to u16.
        (mixed * 0.5 * u16::MAX as f64) as u16
    }
}
