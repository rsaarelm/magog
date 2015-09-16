/*! Audio generation and playing utilities */

//use std::i16;
use std::f64::consts::{PI};
use std::sync::{Arc, Mutex};
//use std::thread;
//use cpal;
use ::{noise, lerp};

const SAMPLE_RATE: u32 = 44100;

/// Play a 44100 Hz mono sound using the sample data from the input
/// iterator. The input sound samples should be in range [-1.0, 1.0].
pub fn play<T: Iterator<Item=f64> + Send + 'static>(_sample: Arc<Mutex<T>>) {
    // TODO: Get this working with the new CPAL API
    unimplemented!();
    /*
    let mut voice = cpal::Voice::new();

    thread::spawn(move || loop {
        {
            let mut wave_data = sample.lock().unwrap();
            let mut buffer = voice.append_data(1, cpal::SamplesRate(SAMPLE_RATE), 32768);
            for sample in buffer.iter_mut() {
                match wave_data.next() {
                    // XXX: https://github.com/tomaka/cpal/issues/39
                    // CPAL samples operate in the i16 space but must be cast to u16.
                    Some(w) => *sample = (w * i16::MAX as f64) as u16,
                    None => return,
                }
            }
        }
        voice.play();
    });
    */
}

pub struct Sample<W> {
    wave: W,
    max_t: Option<usize>,
    t: usize,
}

impl<W: Wave> Sample<W> {
    /// Return an iterator that stops after the given duration.
    pub fn duration(mut self, duration_s: f64) -> Sample<W> {
        self.max_t = Some((duration_s * SAMPLE_RATE as f64) as usize);
        self
    }
}

impl<W: Wave> Iterator for Sample<W> {
    type Item = f64;

    fn next(&mut self) -> Option<f64> {
        if let Some(max_t) = self.max_t {
            if self.t >= max_t { return None; }
        }

        let sample = self.wave.sample(self.t as f64 / SAMPLE_RATE as f64);
        self.t += 1;
        Some(sample)
    }
}

/// Sampleable waveform.
pub trait Wave: Sized {
    /// Get the value of the wave in [-1.0, 1.0] at the given time in seconds.
    fn sample(&self, t: f64) -> f64;

    /// Set the volume of the sound.
    fn volume(self, vol: f64) -> Enveloped<Self, f64> {
        Enveloped { wave: self, envelope: vol }
    }

    /// Set the pitch of the sound.
    fn pitch(self, hz: f64) -> Pitched<Self> {
        Pitched { inner: self, hz: hz }
    }

    /// Give the sound an attack-decay-sustain-release envelope of a given
    /// duration.
    fn adsr(self, duration: f64, atk: f64, dec: f64, sus: f64, rel: f64) -> Enveloped<Self, Adsr> {
        Enveloped {
            wave: self,
            envelope: Adsr {
                attack_s: atk,
                decay_s: dec,
                sustain_volume: sus,
                release_s: rel,
                duration_s: duration,
            }
        }
    }

    /// Turn the sound into a sampling iterator.
    fn into_iter(self) -> Sample<Self> {
        Sample {
            wave: self,
            max_t: None,
            t: 0,
        }
    }
}

pub trait Envelope: Sized {
    fn volume(&self, t: f64) -> f64;
}

/// Different waves at 1 Hz.
pub enum Waveform {
    Sine,
    /// The parameter is the duty cycle of the square wave, in [0.0, 1.0]
    Square(f64),
    Saw,
    Triangle,
    Noise,
}

impl Wave for Waveform {
    fn sample(&self, t: f64) -> f64 {
        use self::Waveform::*;

        let frac = t % 1.0;

        match self {
            &Sine => (frac * PI).sin(),
            &Square(duty) => if frac < duty { 1.0 } else { -1.0 },
            &Saw => -1.0 + frac * 2.0,
            &Triangle => if frac < 0.5 { -1.0 + frac * 4.0 } else { 3.0 - frac * 4.0 },
            &Noise => noise((t * 1024.0) as i32) as f64,
        }
    }
}

pub struct Pitched<W> {
    inner: W,
    hz: f64,
}

impl<W: Wave> Wave for Pitched<W> {
    fn sample(&self, t: f64) -> f64 { self.inner.sample(t * self.hz) }
}

pub struct Enveloped<W, E> {
    wave: W,
    envelope: E,
}

impl<W: Wave, E: Envelope> Wave for Enveloped<W, E> {
    fn sample(&self, t: f64) -> f64 { self.wave.sample(t) * self.envelope.volume(t) }
}

pub struct Adsr {
    attack_s: f64,
    decay_s: f64,
    sustain_volume: f64,
    release_s: f64,
    duration_s: f64,
}

impl Envelope for Adsr {
    fn volume(&self, mut t: f64) -> f64 {
        if t < 0.0 || t > self.duration_s { return 0.0; }

        let sustain_s = self.duration_s - self.attack_s - self.decay_s - self.release_s;

        if t < self.attack_s { return lerp(0.0, 1.0, t / self.attack_s); }
        t -= self.attack_s;

        if t < self.decay_s { return lerp(1.0, self.sustain_volume, t / self.decay_s); }
        t -= self.decay_s;

        if t < sustain_s { return self.sustain_volume; }
        t -= sustain_s;

        lerp(self.sustain_volume, 0.0, t / self.release_s)
    }
}

// Volume adjustment
impl Envelope for f64 {
    fn volume(&self, _: f64) -> f64 { *self }
}

#[derive(Clone)]
pub struct Mixer {
    inner: Arc<Mutex<InnerMixer>>,
}

impl Mixer {
    /// Spawns a new mixer thread and returns a handle to control it.
    pub fn new() -> Mixer {
        let inner = Arc::new(Mutex::new(InnerMixer::new()));
        play(inner.clone());
        Mixer { inner: inner }
    }

    /// Acquire a lock to the inner mixer mutex and add a wave to it.
    pub fn add<T: Iterator<Item=f64>+Send+'static>(&mut self, wave: T) {
        self.inner.lock().unwrap().add(Box::new(wave));
    }

    /// Stop playing the mixer audio.
    pub fn stop(&mut self) {
        self.inner.lock().unwrap().running = false;
    }
}

struct InnerMixer {
    waves: Vec<Box<Iterator<Item=f64> + Send>>,
    running: bool,
}

impl InnerMixer {
    fn new() -> InnerMixer {
        InnerMixer {
            waves: Vec::new(),
            running: true,
        }
    }

    fn add(&mut self, wave: Box<Iterator<Item=f64> + Send>) {
        self.waves.push(wave);
    }
}

impl Iterator for InnerMixer {
    type Item = f64;

    fn next(&mut self) -> Option<f64> {
        if !self.running { return None; }

        let mut ret = 0.0;
        // Indexes of ended waves.
        let mut deletes = Vec::new();

        // Reverse the range because we want indices going to deletes vector
        // in reverse order.
        for i in (0..(self.waves.len())).rev() {
            match self.waves[i].next() {
                None => { deletes.push(i); }
                Some(a) => {
                    // Do some hacky equalization to keep summed waves from
                    // going outside the wave envelope.
                    ret = a + ret - a.signum() * (a * ret).max(0.0);
                }
            }
        }

        // Because deletes is in reverse order, we can swap-remove everything
        // we come across and not invalidate the values further in the list.
        for &del_idx in deletes.iter() {
            self.waves.swap_remove(del_idx);
        }

        Some(ret)
    }
}
