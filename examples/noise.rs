extern crate calx;
use calx::audio::{self, Wave, Waveform};
use std::thread;
use std::time::Duration;

fn main() {
    let mut mixer = audio::Mixer::new();
    mixer.add(Waveform::Noise.volume(0.02)
                             .into_iter()
                             .duration(2.0));
    thread::sleep(Duration::from_secs(1));
    mixer.add(Waveform::Sine.pitch(500.0)
                            .adsr(2.0, 0.2, 0.01, 0.3, 0.2)
                            .volume(0.5)
                            .into_iter()
                            .duration(2.0));
    thread::sleep(Duration::from_secs(1));
}
