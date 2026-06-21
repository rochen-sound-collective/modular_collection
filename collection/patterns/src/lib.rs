mod active_note;
#[cfg(feature = "gui")]
mod editor;
#[cfg(feature = "gui")]
mod note_viewer;
mod processors;
mod utils;

use crate::processors::ChordPatternProcessor;
use crate::utils::{default_expr_value, new_expr_event, set_note_voice_channel_of_event, KeyboardMode};
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum ExprType {
    Pressure,
    Volume,
    Pan,
    Tuning,
    Vibrato,
    Expression,
    Brightness,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct ExprKey {
    pub voice_id: i64, // -1 if None
    pub channel: u8,
    pub note: u8,
    pub kind: ExprType,
}

#[derive(Enum, Debug, PartialEq)]
pub enum PlayMode {
    #[id = "pattern"]
    #[name = "Pattern"]
    Pattern,
    #[id = "chord_pattern"]
    #[name = "Chord Pattern"]
    ChordPattern,
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

    #[id = "expr_mix"]
    expr_mix: FloatParam,

    // Removed: expression smoothing parameter
    #[id = "play_mode"]
    play_mode: EnumParam<PlayMode>,
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
            expr_mix: FloatParam::new(
                "Expression Mix",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            play_mode: EnumParam::new("Play Mode", PlayMode::Pattern),
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

    fn mixed_expr_for_pattern(&self, pd: &crate::processors::PatternData, kind: ExprType) -> f32 {
        let mix = self.params.expr_mix.value();
        let base_note_opt = self
            .processor
            .chord
            .iter()
            .nth(pd.chord_idx() as usize)
            .copied();
        let p_opt = pd.expr_value(kind);
        let c_opt = if let Some(base) = base_note_opt {
            self.processor
                .chord_expr
                .get(&base)
                .and_then(|st| st.get(kind))
        } else {
            None
        };
        let (p_v, c_v) = match (p_opt, c_opt) {
            (Some(p), Some(c)) => (p, c),
            (Some(p), None) => (p, p),
            (None, Some(c)) => (c, c),
            (None, None) => {
                let d = default_expr_value(kind);
                (d, d)
            }
        };
        p_v * (1.0 - mix) + c_v * mix
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

    // Enable basic note event IO; CCs and other messages can still be handled as PluginNoteEvent variants
    // while ensuring note and note-expression events are passed through by the host.
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

            let mut other_events: Vec<PluginNoteEvent<Patterns>> = Vec::new();

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
                        self.params.play_mode.value(),
                    );

                    // Periodic expression emission at this boundary
                    self.processor.maybe_emit_expressions(
                        note_events,
                        sample_id,
                        self.params.expr_mix.value(),
                    );
                    for note_event in note_events {
                        context.send_event(note_event.clone());
                    }
                    other_events.clear();
                    sample_id = event.timing();
                }

                let note_channel = utils::get_channel_of_event::<Self>(&event);

                if note_channel == Some((self.params.chord_channel.value() - 1) as u8) {
                    match &event {
                        PluginNoteEvent::<Patterns>::NoteOn { .. }
                        | PluginNoteEvent::<Patterns>::NoteOff { .. } => {
                            self.processor.process_chord_event(event.clone());
                        }
                        PluginNoteEvent::<Patterns>::Choke { note, .. } => {
                            // Immediate choke fanout
                            let chord_note = *note;
                            for (_raw, pd) in self.processor.held_pattern_keys.iter() {
                                if let Some(&base) =
                                    self.processor.chord.iter().nth(pd.chord_idx() as usize)
                                {
                                    if base == chord_note {
                                        if let Some(tn) = pd.triggered_note() {
                                            let routed = set_note_voice_channel_of_event::<Self>(
                                                &event,
                                                tn,
                                                pd.voice_id(),
                                                pd.channel(),
                                            );
                                            context.send_event(routed);
                                        }
                                    }
                                }
                            }
                        }
                        PluginNoteEvent::<Patterns>::PolyVolume { .. }
                        | PluginNoteEvent::<Patterns>::PolyPan { .. }
                        | PluginNoteEvent::<Patterns>::PolyTuning { .. }
                        | PluginNoteEvent::<Patterns>::PolyVibrato { .. }
                        | PluginNoteEvent::<Patterns>::PolyExpression { .. }
                        | PluginNoteEvent::<Patterns>::PolyBrightness { .. }
                        | PluginNoteEvent::<Patterns>::MidiChannelPressure { .. }
                        | PluginNoteEvent::<Patterns>::MidiCC { .. } => {
                            self.processor.update_chord_expr_from_event(&event);
                            // Also propagate channel events across chord as base values
                            self.processor.update_chord_expr_from_channel_event(&event);
                        }
                        _ => {}
                    }
                } else {
                    match event {
                        // On pattern-side PolyPressure, immediately emit both PolyPressure
                        // (retargeted to the triggered note) and Channel Pressure for compatibility
                        PluginNoteEvent::<Patterns>::PolyPressure { timing, .. } => {
                            // Update expression states as usual
                            self.processor.update_pattern_expr_from_event(
                                &event,
                                &self.params.key_mode.value(),
                            );
                            self.processor
                                .update_pattern_expr_from_channel_event(&event);

                            // Try to route this poly pressure to the currently triggered note
                            if let Some(raw_note) = utils::get_note_of_event::<Self>(&event)
                                .and_then(|n| {
                                    utils::raw_note_apply_keyboard_mode(
                                        n,
                                        &self.params.key_mode.value(),
                                    )
                                })
                            {
                                if let Some(pd) = self.processor.held_pattern_keys.get(&raw_note) {
                                    if let Some(tn) = pd.triggered_note() {
                                        // Compute mixed pressure using current chord + pattern values
                                        let mix = self.params.expr_mix.value();
                                        let base_note_opt = self
                                            .processor
                                            .chord
                                            .iter()
                                            .nth(pd.chord_idx() as usize)
                                            .copied();
                                        let p_opt = pd.expr_value(crate::ExprType::Pressure);
                                        let c_opt = if let Some(base) = base_note_opt {
                                            self.processor
                                                .chord_expr
                                                .get(&base)
                                                .and_then(|st| st.get(crate::ExprType::Pressure))
                                        } else {
                                            None
                                        };
                                        let (p_v, c_v) = match (p_opt, c_opt) {
                                            (Some(p), Some(c)) => (p, c),
                                            (Some(p), None) => (p, p),
                                            (None, Some(c)) => (c, c),
                                            (None, None) => {
                                                let d = default_expr_value(crate::ExprType::Pressure);
                                                (d, d)
                                            }
                                        };
                                        let mixed = p_v * (1.0 - mix) + c_v * mix;

                                        // Emit mixed PolyPressure retargeted to triggered note
                                        let poly = new_expr_event::<Self>(
                                            crate::ExprType::Pressure,
                                            timing,
                                            tn,
                                            pd.voice_id(),
                                            pd.channel(),
                                            mixed,
                                        );
                                        context.send_event(poly);

                                        // Also emit channel pressure on the same channel using mixed value
                                        context.send_event(NoteEvent::MidiChannelPressure {
                                            timing: timing,
                                            channel: pd.channel(),
                                            pressure: mixed,
                                        });
                                    }
                                }
                            }
                        }
                        // Immediate mixed forwarding for other per-note expression types
                        PluginNoteEvent::<Patterns>::PolyVolume { timing, .. }
                        | PluginNoteEvent::<Patterns>::PolyPan { timing, .. }
                        | PluginNoteEvent::<Patterns>::PolyTuning { timing, .. }
                        | PluginNoteEvent::<Patterns>::PolyVibrato { timing, .. }
                        | PluginNoteEvent::<Patterns>::PolyExpression { timing, .. }
                        | PluginNoteEvent::<Patterns>::PolyBrightness { timing, .. } => {
                            // Update expression states
                            self.processor.update_pattern_expr_from_event(
                                &event,
                                &self.params.key_mode.value(),
                            );
                            self.processor
                                .update_pattern_expr_from_channel_event(&event);

                            if let Some(raw_note) = utils::get_note_of_event::<Self>(&event)
                                .and_then(|n| {
                                    utils::raw_note_apply_keyboard_mode(
                                        n,
                                        &self.params.key_mode.value(),
                                    )
                                })
                            {
                                if let Some(pd) = self.processor.held_pattern_keys.get(&raw_note) {
                                    if let Some(tn) = pd.triggered_note() {
                                        // Determine kind from the event variant and emit
                                        let kind = match &event {
                                            PluginNoteEvent::<Patterns>::PolyVolume { .. } => ExprType::Volume,
                                            PluginNoteEvent::<Patterns>::PolyPan { .. } => ExprType::Pan,
                                            PluginNoteEvent::<Patterns>::PolyTuning { .. } => ExprType::Tuning,
                                            PluginNoteEvent::<Patterns>::PolyVibrato { .. } => ExprType::Vibrato,
                                            PluginNoteEvent::<Patterns>::PolyExpression { .. } => ExprType::Expression,
                                            PluginNoteEvent::<Patterns>::PolyBrightness { .. } => ExprType::Brightness,
                                            _ => unreachable!(),
                                        };
                                        let mixed = self.mixed_expr_for_pattern(pd, kind);
                                        let poly = new_expr_event::<Self>(
                                            kind,
                                            timing,
                                            tn,
                                            pd.voice_id(),
                                            pd.channel(),
                                            mixed,
                                        );
                                        context.send_event(poly);
                                    }
                                }
                            }
                        }
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
                        PluginNoteEvent::<Patterns>::MidiChannelPressure { .. }
                        | PluginNoteEvent::<Patterns>::MidiCC { .. } => {
                            self.processor.update_pattern_expr_from_event(
                                &event,
                                &self.params.key_mode.value(),
                            );
                            self.processor
                                .update_pattern_expr_from_channel_event(&event);
                        }
                        _ => {}
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
                self.params.play_mode.value(),
            );

            // Final periodic expression emission for this block and send
            self.processor.maybe_emit_expressions(
                note_events,
                sample_id,
                self.params.expr_mix.value(),
            );
            for event_to_send in note_events {
                context.send_event(event_to_send.clone());
            }
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
