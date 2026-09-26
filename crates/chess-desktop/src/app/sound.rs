//! Short cues synthesised from sine tones, so there are no sample files.

use rodio::source::{SineWave, Source};
use rodio::{DeviceSinkBuilder, MixerDeviceSink};
use std::time::Duration;

/// What just happened, as far as the ear is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cue {
    Move,
    Capture,
    Check,
    GameOver,
    /// One tick per second under ten seconds on the clock.
    LowTime,
}

/// The default output device, or nothing on a machine without one.
pub struct Sound {
    sink: Option<MixerDeviceSink>,
}

impl Sound {
    pub fn open() -> Self {
        let sink = DeviceSinkBuilder::open_default_sink().ok().map(|mut sink| {
            sink.log_on_drop(false);
            sink
        });
        Self { sink }
    }

    /// True when a device could be opened.
    pub fn is_available(&self) -> bool {
        self.sink.is_some()
    }

    pub fn play(&self, cue: Cue) {
        let Some(sink) = &self.sink else {
            return;
        };
        let tone = |hz: f32, ms: u64, gain: f32, after_ms: u64| {
            let length = Duration::from_millis(ms);
            // A quieter octave on top rounds off the bare sine; the short
            // fade-in stops the first sample from clicking.
            SineWave::new(hz)
                .mix(SineWave::new(hz * 2.0).amplify(0.3))
                .take_duration(length)
                .fade_in(Duration::from_millis(4))
                .fade_out(length)
                .amplify(gain)
                .delay(Duration::from_millis(after_ms))
        };
        let mixer = sink.mixer();
        match cue {
            Cue::Move => mixer.add(tone(523.0, 50, 0.2, 0)),
            Cue::Capture => {
                mixer.add(tone(523.0, 60, 0.22, 0));
                mixer.add(tone(392.0, 80, 0.22, 50));
            }
            Cue::Check => mixer.add(tone(784.0, 140, 0.22, 0)),
            Cue::GameOver => {
                mixer.add(tone(523.0, 150, 0.22, 0));
                mixer.add(tone(440.0, 150, 0.22, 150));
                mixer.add(tone(349.0, 300, 0.22, 300));
            }
            Cue::LowTime => mixer.add(tone(1000.0, 25, 0.12, 0)),
        }
    }
}
