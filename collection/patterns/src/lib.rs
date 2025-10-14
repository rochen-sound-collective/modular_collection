mod active_note;
#[cfg(feature = "gui")]
mod editor;
#[cfg(feature = "gui")]
mod note_viewer;
mod processors;
mod utils;

use crate::processors::ChordPatternProcessor;
use crate::utils::{get_chord_data, get_note_of_event, set_note_of_event, KeyboardMode};
use active_note::ActiveNoteDefaultData;
use nih_plug::prelude::*;
#[cfg(feature = "gui")]
use nih_plug_vizia::ViziaState;
use std::cmp::max;
use std::sync::{Arc, Mutex};

pub struct Patterns {
    params: Arc<PatternsParams>,
    processor: ChordPatternProcessor<Patterns>,
    active_pattern_notes: Arc<Mutex<Vec<ActiveNoteDefaultData>>>,
}

#[derive(Params)]
struct PatternsParams {
    #[cfg(feature = "gui")]
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "chord_channel"]
    chord_channel: IntParam,

    #[id = "wrap_threshold"]
    wrap_threshold: IntParam,

    #[id = "auto_threshold"]
    auto_threshold: BoolParam,

    #[id = "octave_range"]
    octave_range: IntParam,

    #[id = "key_mode"]
    key_mode: EnumParam<KeyboardMode>,

    #[id = "octave_shift"]
    octave_shift: IntParam,
}

impl Default for PatternsParams {
    fn default() -> Self {
        Self {
            #[cfg(feature = "gui")]
            editor_state: editor::default_state(),
            chord_channel: IntParam::new("Chord Channel", 16, IntRange::Linear { min: 1, max: 16 }),
            wrap_threshold: IntParam::new(
                "Wrap Threshold",
                12,
                IntRange::Linear { min: 1, max: 12 },
            ),
            auto_threshold: BoolParam::new("Auto Threshold", true),
            octave_range: IntParam::new("Octave Range", 12, IntRange::Linear { min: 1, max: 127 }),
            key_mode: EnumParam::new("Keyboard Mode", KeyboardMode::AllKeys),
            octave_shift: IntParam::new("Octave Shift", 0, IntRange::Linear { min: -3, max: 3 }),
        }
    }
}

impl Default for Patterns {
    fn default() -> Self {
        Self {
            params: Arc::new(PatternsParams::default()),
            processor: ChordPatternProcessor::<Self>::default(),
            active_pattern_notes: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Patterns {
    fn get_threshold(&self) -> u8 {
        if self.params.auto_threshold.value() {
            max(self.processor.chord.len() as u8, 1) // minimum wrap threshold of 1 to not divide by zero
        } else {
            self.params.wrap_threshold.value() as u8
        }
    }
}

impl Plugin for Patterns {
    const NAME: &'static str = "Modular::Patterns";
    const VENDOR: &'static str = "Rochen Sound Collective";
    const URL: &'static str = "https://github.com/rochen-sound-collective/modular_collection";
    const EMAIL: &'static str = "info@example.com";

    const VERSION: &'static str = "0.0.1";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(0),
            main_output_channels: NonZeroU32::new(0),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(0),
            main_output_channels: NonZeroU32::new(0),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;

    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    const HARD_REALTIME_ONLY: bool = false;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        #[cfg(feature = "gui")]
        {
            return editor::create(
                self.params.clone(),
                self.params.editor_state.clone(),
                self.active_pattern_notes.clone(),
            );
        }
        #[cfg(not(feature = "gui"))]
        {
            None
        }
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        {
            let mut next_event = context.next_event();
            let mut sample_id = 999;
            let current_beats: f32 = context
            .transport()
            .pos_beats()        // Option<f64>
            .unwrap_or(0.0)     // f64
            as f32; // f32

            let mut other_events: Vec<NoteEvent<()>> = Vec::new();

            while let Some(event) = next_event {
                if event.timing() != sample_id {
                    let note_events = &mut vec![];
                    self.processor.end_cycle(
                        note_events,
                        sample_id,
                        self.get_threshold(),
                        self.params.octave_range.value() as u8,
                        self.params.key_mode.value(),
                        self.params.octave_shift.value() as i8,
                    );

                    for note_event in note_events {
                        context.send_event(note_event.clone());
                    }
                    // TODO: Modulate other events too
                    for event in &other_events {
                        context.send_event(event.clone());
                    }
                    other_events.clear();
                    sample_id = event.timing();
                }

                let note_channel = utils::get_channel_of_event::<Self>(&event);

                if note_channel == Some((self.params.chord_channel.value() - 1) as u8) {
                    self.processor.process_chord_event(event.clone());
                } else {
                    match event {
                        PluginNoteEvent::<Patterns>::NoteOn { .. } => {
                            self.processor.process_pattern_event(event.clone());
                            // Add note to active notes
                            let mut active_notes = self.active_pattern_notes.lock().unwrap();
                            let mut note =
                                ActiveNoteDefaultData::from_note_event::<Patterns>(&event);
                            note.start_time_beats = current_beats;

                            active_notes.push(note);
                        }
                        PluginNoteEvent::<Patterns>::NoteOff { note, .. } => {
                            self.processor.process_pattern_event(event);
                            // Set end time and remove note
                            let mut active_notes = self.active_pattern_notes.lock().unwrap();
                            for note_data in active_notes.iter_mut() {
                                if note_data.note == note && note_data.end_time_beats.is_none() {
                                    note_data.end_time_beats = Some(current_beats);
                                    break;
                                }
                            }
                            // Remove notes that have ended after a while
                            active_notes.retain(|n| {
                                n.end_time_beats
                                    .map_or(true, |end| current_beats - end < 2.0)
                            });
                        }
                        _ => other_events.push(event.clone()),
                    }
                }

                next_event = context.next_event();
            }

            let note_events = &mut vec![];
            self.processor.end_cycle(
                note_events,
                sample_id,
                self.get_threshold(),
                self.params.octave_range.value() as u8,
                self.params.key_mode.value(),
                self.params.octave_shift.value() as i8,
            );

            for e in note_events {
                context.send_event(e.clone());
            }
            // TODO: Modulate other events too
            for note_event in &other_events {
                if let Some(raw_note) = get_note_of_event::<Self>(note_event) {
                    let chord_data = get_chord_data(
                        &self.processor.chord.iter().cloned().collect(),
                        raw_note,
                        self.get_threshold(),
                        self.params.octave_range.value() as u8,
                        self.params.octave_shift.value() as i8,
                    );
                    if let Some(triggered_note) = chord_data.triggered_note {
                        context.send_event(set_note_of_event::<Self>(note_event, triggered_note));
                    }
                }
            }

            other_events.clear();
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Patterns {
    const CLAP_ID: &'static str = "com.modular.patterns";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Pattern based arpeggiator.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect, ClapFeature::Utility];
}

impl Vst3Plugin for Patterns {
    const VST3_CLASS_ID: [u8; 16] = *b"modular.patterns";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Tools];
}

nih_export_clap!(Patterns);
nih_export_vst3!(Patterns);

/*
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
*/
