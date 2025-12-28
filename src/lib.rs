use atomic_float::AtomicF32;
use nih_plug::prelude::*;
use std::sync::Arc;
use vizia_plug::ViziaState;

use i_am_dsp::{
    Effect, ProcessContext as DspContext, ProcessInfos, prelude::Disperser,
    real_time_demo::SimpleContext,
};

mod editor;
mod widgets;

const PEAK_METER_DECAY_MS: f64 = 150.0;

pub struct DisperserPlugin {
    params: Arc<DisperserParams>,

    disperser: Disperser<2>,
    sample_rate: f32,

    peak_meter_decay_weight: f32,
    peak_meter: Arc<AtomicF32>,
}

#[derive(Params)]
struct DisperserParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "frequency"]
    pub frequency: FloatParam,

    #[id = "spread"]
    pub spread: FloatParam,

    #[id = "amount"]
    pub amount: IntParam,
}

impl Default for DisperserPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(DisperserParams::default()),
            disperser: Disperser::<2>::new(44100),
            sample_rate: 44100.0,

            peak_meter_decay_weight: 1.0,
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
        }
    }
}

impl Default for DisperserParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            frequency: FloatParam::new(
                "Frequency",
                1145.0,
                FloatRange::Linear {
                    min: 20.0,
                    max: 20000.0,
                },
            )
            .with_unit(" Hz"),

            spread: FloatParam::new(
                "Spread",
                1145.0,
                FloatRange::Linear {
                    min: 0.1,
                    max: 2000.0,
                },
            ),

            amount: IntParam::new("Amount", 80, IntRange::Linear { min: 0, max: 100 }),
        }
    }
}

impl Plugin for DisperserPlugin {
    const NAME: &'static str = "GODIEPERSER";
    const VENDOR: &'static str = "IAMMRGODIE & SOUT AUDIO";
    const URL: &'static str = "https://audio.soout.top/godieperser";
    const EMAIL: &'static str = "sout233@163.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.peak_meter.clone(),
            self.params.editor_state.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        self.disperser = Disperser::<2>::new(self.sample_rate as usize);

        self.peak_meter_decay_weight = 0.25f64
            .powf((buffer_config.sample_rate as f64 * PEAK_METER_DECAY_MS / 1000.0).recip())
            as f32;

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let freq = self.params.frequency.value();
        let spread = self.params.spread.value();
        let amount = self.params.amount.value();

        self.disperser.set_filter_parameters(freq, spread);
        self.disperser.set_biquad_count(amount as usize);

        let mut info = ProcessInfos::new();
        info.sample_rate = self.sample_rate as usize;

        let simple_ctx = SimpleContext {
            info,
            midi_events: Vec::new(),
        };

        let mut dsp_ctx: Box<dyn DspContext> = Box::new(simple_ctx);

        let mut amplitude = 0.0;
        let channels = buffer.channels();

        if channels == 2 {
            let samples = buffer.as_slice();
            let (left_chan, right_chan) = samples.split_at_mut(1);
            let left_samples = &mut left_chan[0];
            let right_samples = &mut right_chan[0];

            for (l, r) in left_samples.iter_mut().zip(right_samples.iter_mut()) {
                let mut frame = [*l, *r];
                let other_inputs: &[&[f32; 2]] = &[];

                self.disperser
                    .process(&mut frame, other_inputs, &mut dsp_ctx);

                *l = frame[0];
                *r = frame[1];

                let current_amp = l.abs().max(r.abs());
                if current_amp > amplitude {
                    amplitude = current_amp;
                }
            }
        }

        for channel_samples in buffer.iter_samples() {
            if self.params.editor_state.is_open() {
                let num_samples = channel_samples.len();
                amplitude = (amplitude / num_samples as f32).abs();
                let current_peak_meter = self.peak_meter.load(std::sync::atomic::Ordering::Relaxed);
                let new_peak_meter = if amplitude > current_peak_meter {
                    amplitude
                } else {
                    current_peak_meter * self.peak_meter_decay_weight
                        + amplitude * (1.0 - self.peak_meter_decay_weight)
                };

                self.peak_meter
                    .store(new_peak_meter, std::sync::atomic::Ordering::Relaxed)
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for DisperserPlugin {
    const CLAP_ID: &'static str = "top.soout.godiedsp.disperser";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Phase Disperser Effect");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Phaser,
    ];
}

impl Vst3Plugin for DisperserPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"GODIEPERSER_SOUT";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Modulation];
}

nih_export_clap!(DisperserPlugin);
nih_export_vst3!(DisperserPlugin);
