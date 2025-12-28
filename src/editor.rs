use atomic_float::AtomicF32;
use nih_plug::prelude::Editor;
use nih_plug::util;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use vizia_plug::vizia::prelude::*;
use vizia_plug::widgets::PeakMeter;
use vizia_plug::{ViziaState, ViziaTheming, create_vizia_editor};

use crate::DisperserParams;
use crate::widgets::params_knob::ParamKnob;
use crate::widgets::waveform_view::WaveformView;

pub const NOTO_SANS: &str = "Noto Sans";

#[derive(Lens)]
struct Data {
    params: Arc<DisperserParams>,
    peak_meter: Arc<AtomicF32>,
    knob_value: f32,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (800, 500))
}

pub(crate) fn create(
    params: Arc<DisperserParams>,
    peak_meter: Arc<AtomicF32>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        // assets::register_noto_sans_light(cx);
        // assets::register_noto_sans_thin(cx);

        cx.add_stylesheet(include_style!("src/style.css"))
            .expect("err when include style.css");
        cx.add_font_mem(include_bytes!("../assets/JetBrainsMono-Bold.ttf"));

        Data {
            params: params.clone(),
            peak_meter: peak_meter.clone(),
            knob_value: 0.0,
        }
        .build(cx);

        VStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    Label::new(cx, "GODIEPERSER").class("top-bar-text");

                    HStack::new(cx, |_| {}).width(Stretch(1.0));

                    Label::new(cx, "PROCESSING").class("top-bar-text");

                    PeakMeter::new(
                        cx,
                        Data::peak_meter
                            .map(|peak_meter| util::gain_to_db(peak_meter.load(Ordering::Relaxed))),
                        Some(Duration::from_millis(600)),
                    );
                })
                .padding(Pixels(4.0))
                .height(Pixels(20.0));
                VStack::new(cx, |cx| {
                    WaveformView::new(
                        cx,
                        Data::peak_meter.map(|peak_meter| peak_meter.load(Ordering::Relaxed)),
                        512,
                    )
                    .class("waveform-view");
                })
                .height(Stretch(1.0));
            })
            .class("spectrum-panel");

            VStack::new(cx, |cx| {
                VStack::new(cx, |_cx| {})
                    .width(Percentage(100.0))
                    .class("shadow-bar");

                HStack::new(cx, |cx| {
                    VStack::new(cx, |cx| {
                        Label::new(cx, "GODIEPERSER")
                            .font_size(24.0)
                            .background_color(Color::rgb(18, 23, 19))
                            .color(Color::rgb(243, 255, 244));
                        Label::new(cx, "DSP CORE BY IAMMRDODIE")
                            .font_size(12.0)
                            .color(Color::rgb(18, 23, 19));
                    })
                    .gap(Pixels(4.0))
                    .padding_left(Pixels(48.0))
                    .alignment(Alignment::Left);

                    HStack::new(cx, |_| {}).width(Stretch(1.0));

                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            ParamKnob::new(cx, Data::params, |params| &params.amount, true)
                                .class("knob");
                            Label::new(cx, "AMOUNT").class("params-label");
                        })
                        .class("knob-cont");

                        VStack::new(cx, |cx| {
                            ParamKnob::new(cx, Data::params, |params| &params.spread, true)
                                .class("knob");
                            Label::new(cx, "SPREAD").class("params-label");
                        })
                        // genshin impact is the worst game in the world
                        .class("knob-cont");

                        VStack::new(cx, |cx| {
                            ParamKnob::new(cx, Data::params, |params| &params.frequency, true)
                                .class("knob");
                            Label::new(cx, "FREQUENCY").class("params-label");
                        })
                        .class("knob-cont");
                    })
                    .padding_right(Pixels(48.0))
                    .alignment(Alignment::Right)
                    .gap(Pixels(24.0));
                })
                .height(Stretch(1.0));
            })
            .class("control-panel");
        });

        // VStack::new(cx, |cx| {
        //     Label::new(cx, "GODIEPERSER")
        //         .font_family(vec![FamilyOwned::Named(String::from(NOTO_SANS))])
        //         .font_weight(FontWeightKeyword::Light)
        //         .font_size(30.0)
        //         .height(Pixels(50.0))
        //         .alignment(Alignment::BottomCenter);

        //     Label::new(cx, "Amount");
        //     ParamSlider::new(cx, Data::params, |params| &params.amount);

        //     PeakMeter::new(
        //         cx,
        //         Data::peak_meter
        //             .map(|peak_meter| util::gain_to_db(peak_meter.load(Ordering::Relaxed))),
        //         Some(Duration::from_millis(600)),
        //     );
        // })
        // .alignment(Alignment::TopCenter);
    })
}
