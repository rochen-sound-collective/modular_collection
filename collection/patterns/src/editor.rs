use std::collections::BTreeMap;
use nih_plug::prelude::{util, Editor};
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use crate::active_note::ActiveNoteDefaultData;
//use nih_plug_vizia::vizia::image::error::UnsupportedErrorKind::Color;

use crate::PatternsParams;

const STYLE: &str = r#"
    .spacer {
        background-color: #526A68;
    }

    .label-header {
        font-size: 28;
        left: 1s;
        right: 1s;
        child-bottom: 0px;
    }

    vstack{
        row-between: 0.0px;
        child-left: 1.0s;
        child-right: 1.0s;
        top: 1.0s;
        bottom: 1.0s;
    }
    param-slider {
        background-color: #7EA29F;
    }
    param-slider .value {
        color: #FFFFFF;
    }

    param-slider .fill{
        background-color: #2F4140;
    }

    .chord-view{
        height: 64px;
        row-between: 0.0;
        child-left: 20px;
        child-right: 20px;
        left: 1s;
        right: 1s;
        background-color: #2F4140;
        outline-color: #2F4140;
        outline-width: 10;
        margin: 10;
    }

    .label-chord-key{
        outer-shadow: 0 1 1 transparent;
        font-size: 32;
        left: 0.5s;
        right: 0.5s;
        top: 1s;
        child-bottom: 0px;
        color: #FFFFFF;
    }

    .label-chord-detail{
        outer-shadow: 0 1 1 transparent;
        font-style: italic;
        font-size: 24;
        left: 1s;
        right: 1s;
        top: 1s;
        child-bottom: 0px;
        color: #FFFFFF;
    }
"#;

#[derive(Lens)]
struct Data {
    params: Arc<PatternsParams>,
    note_cache: BTreeMap<u8, ActiveNoteDefaultData>
}

impl Model for Data {}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::from_size(800, 600)
}

pub(crate) fn create(
    params: Arc<PatternsParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        cx.add_theme(STYLE);

        Data {
            params: params.clone(),
            note_cache: Default::default()
        }
        .build(cx);

        ResizeHandle::new(cx);

        HStack::new(cx, |cx| {
            Element::new(cx)
                .height(Stretch(1.0))
                .width(Stretch(1.0));
            Element::new(cx)
                .height(Stretch(1.0))
                .width(Pixels(3.0))
                .class("spacer");
            VStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    Label::new(cx, "D")
                        .class("label-chord-key");

                    Label::new(cx, "min 7 / C")
                        .class("label-chord-detail");
                })
                .class("chord-view");

                VStack::new(cx, |cx| {
                    Label::new(cx, "MIDI Input")
                        .class("label-header");

                    // NOTE: VIZIA adds 1 pixel of additional height to these labels, so we'll need to
                    //       compensate for that
                    Label::new(cx, "Chord channel").bottom(Pixels(-1.0));
                    ParamSlider::new(cx, Data::params, |params| &params.chord_channel);
                });


                VStack::new(cx, |cx| {
                    Label::new(cx, "Octave wrapping")
                        .class("label-header");

                    // NOTE: VIZIA adds 1 pixel of additional height to these labels, so we'll need to
                    //       compensate for that
                    Label::new(cx, "Align to chord").bottom(Pixels(-1.0));
                    ParamSlider::new(cx, Data::params, |params| &params.auto_threshold);

                    Label::new(cx, "Wrap threshold").bottom(Pixels(-1.0));
                    ParamSlider::new(cx, Data::params, |params| &params.wrap_threshold);

                    Label::new(cx, "Octave range").bottom(Pixels(-1.0));
                    ParamSlider::new(cx, Data::params, |params| &params.octave_range);
                });
            })
            .width(Stretch(0.3))
            .row_between(Pixels(0.0))
            .child_left(Pixels(10.0))
            .child_right(Pixels(10.0));
        });
    })
}
