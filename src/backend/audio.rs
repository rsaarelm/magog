use std::u16;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use num::traits::{Float};
use time;
use cpal;

/// A handle object for playing sounds.
#[derive(Clone)]
pub struct Mixer {
    tx: Arc<Mutex<mpsc::Sender<Rpc>>>,
}

impl Mixer {
    /// Create a new mixer and spawn a sound-playing thread that will persist
    /// for the lifetime of the mixer.
    pub fn new() -> Mixer {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut engine = Engine::new();
            let mut voice = cpal::Voice::new();

            loop {
                match rx.try_recv() {
                    Ok(rpc) => {
                        engine.recv(rpc);
                    }
                    // When Mixer is destroyed, the sender object will drop
                    // and trying to receive will get a disconnected error.
                    // This will kill the thread.
                    Err(mpsc::TryRecvError::Disconnected) => { break; }
                    Err(mpsc::TryRecvError::Empty) => {}
                }

                engine.fill_buffer(&mut voice);
                voice.play();
            }
        });

        Mixer {
            tx: Arc::new(Mutex::new(tx)),
        }
    }

    /// Play a sound described by the given wave function and with the given
    /// duration.
    pub fn add_wave(&mut self, f: Box<Fn(f64) -> f64 + Send>, duration: f64) {
        assert!(duration >= 0.0);
        let tx = self.tx.lock().unwrap();
        tx.send(Rpc::Wave((f, duration))).unwrap();
    }
}

enum Rpc {
    Wave((Box<Fn(f64) -> f64 + Send>, f64)),
}

/// State in the thread
struct Engine {
    waves: Vec<Wave>,
    t: f64,
    rate: u32,
}

struct Wave {
    f: Box<Fn(f64) -> f64 + Send>,
    begin_t: f64,
    end_t: f64,
}

impl Engine {
    fn new() -> Engine {
        Engine {
            waves: Vec::new(),
            t: time::precise_time_s(),
            rate: 44100,
        }
    }

    fn recv(&mut self, rpc: Rpc) {
        match rpc {
            Rpc::Wave((f, duration)) => {
                let t = time::precise_time_s();
                self.waves.push(Wave {
                    f: f,
                    begin_t: t,
                    end_t: t + duration
                });
            }
        }
    }

    fn fill_buffer(&mut self, voice: &mut cpal::Voice) {
        let mut buffer =
            voice.append_data(1, cpal::SamplesRate(self.rate), 32768);

        for sample in buffer.iter_mut() {
            *sample = self.tick();
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
