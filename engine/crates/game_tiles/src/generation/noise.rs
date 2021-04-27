use game_lib::rand::Rng;
use std::{f32::consts::TAU, ops::RangeInclusive};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Wave {
    amplitude: f32,
    wavelength: f32,
    phase: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Waves(Vec<Wave>);

#[derive(Clone, PartialEq, Debug)]
pub struct WavesConfig {
    pub waves: RangeInclusive<usize>,
    pub amplitude: RangeInclusive<f32>,
    pub wavelength: RangeInclusive<f32>,
    pub phase: RangeInclusive<f32>,
}

impl Waves {
    pub fn new() -> Self {
        Vec::new().into()
    }

    pub fn new_rand<R: Rng>(rand: &mut R, config: WavesConfig) -> Self {
        let WavesConfig {
            waves,
            amplitude,
            wavelength,
            phase,
        } = config;

        waves
            .map(|_| Wave {
                amplitude: rand.gen_range(amplitude.clone()),
                wavelength: rand.gen_range(wavelength.clone()),
                phase: rand.gen_range(phase.clone()),
            })
            .collect::<Vec<_>>()
            .into()
    }

    pub fn get(&self, x: f32) -> f32 {
        self.0.iter().fold(0.0, |acc, wave| {
            let Wave {
                amplitude,
                wavelength,
                phase,
            } = wave;

            let x = x as f32;
            let offset = amplitude * f32::sin(TAU / wavelength * x + phase);
            acc + offset
        })
    }
}

impl From<Vec<Wave>> for Waves {
    fn from(value: Vec<Wave>) -> Self {
        Waves(value)
    }
}
