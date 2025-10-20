/*!
This module implements the processing of MIDI chord and pattern events for a modular pattern-based
arpeggiator plugin. The module defines data structures used for representing chord data corresponding to
incoming note events and provides logic for transforming pattern events into modulated note events based
on the current chord state.

The main components include:
- `PatternChordData`: Represents chord mapping information such as chord index, octave offset, and an optional
  triggered note.
- `PatternData`: Combines chord mapping with active note data to represent a note's transformation.
- `ChordPatternProcessor`: Processes incoming chord and pattern events, updates internal state, and generates
  corresponding modulated MIDI note events.

This module relies on utilities defined in the `utils` module and the `ActiveNoteDefaultData` structure from the
`active_note` module.
*/

use crate::active_note::{ActiveNoteDefaultData, ExpressionState};
use crate::utils::{
    default_expr_value, get_chord_data_from_set, get_note_of_event, new_expr_event,
    raw_note_apply_keyboard_mode, try_get_expr_type_value, KeyboardMode,
};
use crate::ExprType;
use nih_plug::midi::NoteEvent::{NoteOff, NoteOn};
use nih_plug::midi::PluginNoteEvent;
use nih_plug::midi::control_change as cc;
use nih_plug::prelude::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Represents the chord mapping data for a note in the context of a chord pattern.
///
/// # Fields
/// - `chord_idx`: The index into the chord/pattern array indicating which chord note should be used.
/// - `octave`: The octave offset to be applied. May be negative if the note is below the base octave.
/// - `triggered_note`: Optionally, the actual MIDI note number to trigger based on the chord mapping.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Default)]
pub struct PatternChordData {
    pub chord_idx: u8,
    pub octave: i8,
    pub triggered_note: Option<u8>,
}

/// Combines the chord mapping data with active note data for a pattern note.
///
/// This structure stores the transformation information for a single note event.
/// It holds both the chord data (mapping details) and the original note data obtained from
/// an instance of `ActiveNoteDefaultData`.
pub struct PatternData {
    chord_data: PatternChordData,
    note_data: ActiveNoteDefaultData,
}

impl PatternData {
    pub fn triggered_note(&self) -> Option<u8> {
        self.chord_data.triggered_note
    }

    pub fn chord_idx(&self) -> u8 {
        self.chord_data.chord_idx
    }
    /// Generates a MIDI NoteOn event for this pattern note if a triggered note is available.
    ///
    /// The function uses the chord mapping (`triggered_note`) to construct a new NoteOn event.
    ///
    /// # Type Parameters
    /// - `P`: The plugin type which implements `nih_plug::prelude::Plugin`.
    ///
    /// # Parameters
    /// - `timing`: The sample-accurate timing at which the event should occur.
    ///
    /// # Returns
    /// - `Some(NoteEvent)` if a triggered note is mapped, or `None` if not.
    pub fn note_on<P: nih_plug::prelude::Plugin>(&self, timing: u32) -> Option<PluginNoteEvent<P>> {
        if let Some(modulated_note) = self.chord_data.triggered_note {
            Some(NoteOn {
                note: modulated_note,
                channel: self.note_data.channel,
                velocity: self.note_data.velocity,
                voice_id: self.note_data.voice_id,
                timing,
            })
        } else {
            None
        }
    }

    /// Generates a MIDI NoteOff event for this pattern note if a triggered note is available.
    ///
    /// This function creates a NoteOff event based on the chord mapping, effectively releasing the note.
    ///
    /// # Type Parameters
    /// - `P`: The plugin type which implements `nih_plug::prelude::Plugin`.
    ///
    /// # Parameters
    /// - `timing`: The sample-accurate timing at which the event should occur.
    ///
    /// # Returns
    /// - `Some(NoteEvent)` if a triggered note is mapped, or `None` if not.
    pub fn note_off<P: nih_plug::prelude::Plugin>(
        &self,
        timing: u32,
    ) -> Option<PluginNoteEvent<P>> {
        if let Some(modulated_note) = self.chord_data.triggered_note {
            Some(NoteOff {
                note: modulated_note,
                channel: self.note_data.channel,
                velocity: self.note_data.velocity,
                voice_id: self.note_data.voice_id,
                timing,
            })
        } else {
            None
        }
    }

    pub fn voice_id(&self) -> Option<i32> {
        self.note_data.voice_id
    }

    pub fn channel(&self) -> u8 {
        self.note_data.channel
    }

    pub fn expr_value(&self, kind: crate::ExprType) -> Option<f32> {
        self.note_data.expression.get(kind)
    }
}

/// Processes MIDI chord and pattern events to transform them into modulated note events.
///
/// This processor maintains internal state for pressed and released pattern keys as well as the
/// current chord (represented as a set of active chord note numbers). It supports:
/// - Queueing of pressed pattern keys for later processing.
/// - Queueing of released pattern keys to send NoteOff events.
/// - Maintaining held pattern keys and updating their chord mapping based on changes to the chord state.
/// - Processing raw chord events to update the chord set.
///
/// # Type Parameters
/// - `P`: The plugin type parameter that implements `nih_plug::prelude::Plugin`.
#[derive(Default)]
pub struct ChordPatternProcessor<P: nih_plug::prelude::Plugin> {
    /// Queue of pressed pattern note events awaiting processing.
    pub pressed_pattern_keys: VecDeque<PluginNoteEvent<P>>,
    /// Queue of released pattern note events awaiting processing.
    pub released_pattern_keys: VecDeque<PluginNoteEvent<P>>,
    /// Map of currently held pattern keys, mapping raw note values to their corresponding pattern data.
    pub held_pattern_keys: BTreeMap<u8, PatternData>,
    /// The current chord state represented as a set of active MIDI note numbers.
    pub chord: BTreeSet<u8>,
    /// Chord-side expression smoothers keyed by base chord note
    pub chord_expr: BTreeMap<u8, ExpressionState>,
    /// Expression emission scheduling interval (in samples)
    pub expr_tick_interval: u32,
    /// Last sample time when expressions were emitted
    pub last_expr_emit_sample: u32,
}

impl<P: nih_plug::prelude::Plugin> ChordPatternProcessor<P> {
    /*
        The following function processes the pending changes for pattern events.
        It handles released keys, updates chord changes for held keys, and processes newly
        pressed keys.
    */

    /// Applies pending changes to pattern events, including handling releases, updating held keys,
    /// and processing newly pressed keys.
    ///
    /// The function performs its tasks in three stages:
    /// 1. **Released Keys:** Processes all queued released pattern keys and sends a NoteOff event
    ///    if a corresponding held key exists with an active triggered note.
    /// 2. **Chord Changes for Held Keys:** Iterates over held pattern keys, recalculates their chord
    ///    mapping, and if the mapping has changed, sends a NoteOff for the previous mapping and a NoteOn
    ///    for the new mapping.
    /// 3. **Pressed Keys:** Processes all queued pressed pattern keys by mapping them to chord notes,
    ///    generating NoteOn events, and storing them in the held keys map.
    ///
    /// # Parameters
    /// - `send_events`: A mutable vector into which all generated MIDI events are appended.
    /// - `timing`: The sample-accurate timing for the events produced in this cycle.
    /// - `wrap_threshold`: Determines the modular wrap-around value for chord note mapping.
    /// - `octave_range`: The scaling factor for the octave offset for the triggered note.
    /// - `keyboard_mode`: Specifies how to process the raw note (e.g., ignoring black keys).
    fn apply_pattern_changes(
        &mut self,
        send_events: &mut Vec<PluginNoteEvent<P>>,
        timing: u32,
        wrap_threshold: u8,
        octave_range: u8,
        keyboard_mode: KeyboardMode,
        octave_shift: i8,
    ) {
        self.process_released_keys(send_events, &keyboard_mode);

        self.process_chord_changes(
            send_events,
            timing,
            wrap_threshold,
            octave_range,
            octave_shift,
        );

        self.process_pressed_keys(
            send_events,
            wrap_threshold,
            octave_range,
            keyboard_mode,
            octave_shift,
        );
    }

    fn process_pressed_keys(
        &mut self,
        send_events: &mut Vec<PluginNoteEvent<P>>,
        wrap_threshold: u8,
        octave_range: u8,
        keyboard_mode: KeyboardMode,
        octave_shift: i8,
    ) {
        while let Some(note_event) = self.pressed_pattern_keys.pop_back() {
            if let Some(raw_note) = get_note_of_event::<P>(&note_event)
                .and_then(|note| raw_note_apply_keyboard_mode(note, &keyboard_mode))
            {
                let chord_data = get_chord_data_from_set(
                    &self.chord,
                    raw_note,
                    wrap_threshold,
                    octave_range,
                    octave_shift,
                );

                let active_note = PatternData {
                    chord_data,
                    note_data: ActiveNoteDefaultData::from_note_event::<P>(&note_event),
                };

                if let Some(modulated_event) = active_note.note_on::<P>(note_event.timing()) {
                    send_events.push(modulated_event);
                }
                self.held_pattern_keys.insert(raw_note, active_note);
            }
        }
    }

    fn process_released_keys(
        &mut self,
        send_events: &mut Vec<PluginNoteEvent<P>>,
        keyboard_mode: &KeyboardMode,
    ) {
        while let Some(note_event) = self.released_pattern_keys.pop_back() {
            if let Some(raw_note) = get_note_of_event::<P>(&note_event)
                .and_then(|note| raw_note_apply_keyboard_mode(note, keyboard_mode))
            {
                if let Some(pattern_data) = self.held_pattern_keys.remove(&raw_note) {
                    if let Some(modulated_event) = pattern_data.note_off::<P>(note_event.timing()) {
                        send_events.push(modulated_event);
                    }
                }
            }
        }
    }

    fn process_chord_changes(
        &mut self,
        send_events: &mut Vec<PluginNoteEvent<P>>,
        timing: u32,
        wrap_threshold: u8,
        octave_range: u8,
        octave_shift: i8,
    ) {
        // Process chord changes and octave shifts for held keys
        for (raw_note, pattern_data) in self.held_pattern_keys.iter_mut() {
            let chord_data = get_chord_data_from_set(
                &self.chord,
                *raw_note,
                wrap_threshold,
                octave_range,
                octave_shift,
            );
            if pattern_data.chord_data != chord_data {
                if let Some(modulated_event) = pattern_data.note_off::<P>(timing) {
                    send_events.push(modulated_event);
                }
                pattern_data.chord_data = chord_data;
                if let Some(modulated_event) = pattern_data.note_on::<P>(timing) {
                    send_events.push(modulated_event);
                }
            }
        }
    }

    pub fn update_pattern_expr_from_event(
        &mut self,
        note_event: &PluginNoteEvent<P>,
        keyboard_mode: &KeyboardMode,
    ) {
        if let Some((kind, value)) = try_get_expr_type_value::<P>(note_event) {
            if let Some(raw_note) = get_note_of_event::<P>(note_event)
                .and_then(|n| raw_note_apply_keyboard_mode(n, keyboard_mode))
            {
                if let Some(pattern_data) = self.held_pattern_keys.get_mut(&raw_note) {
                    *pattern_data
                        .note_data
                        .expression
                        .value_mut(kind) = Some(value);
                }
            }
        }
    }

    pub fn update_chord_expr_from_event(&mut self, note_event: &PluginNoteEvent<P>) {
        if let Some((kind, value)) = try_get_expr_type_value::<P>(note_event) {
            if let Some(chord_note) = get_note_of_event::<P>(note_event) {
                let entry = self
                    .chord_expr
                    .entry(chord_note)
                    .or_insert_with(ExpressionState::default);
                *entry.value_mut(kind) = Some(value);
            }
        }
    }

    pub fn update_pattern_expr_from_channel_event(&mut self, event: &PluginNoteEvent<P>) {
        match event {
            NoteEvent::MidiChannelPressure { channel, pressure, .. } => {
                let ch = *channel;
                let val = *pressure;
                for (_raw, pattern_data) in self.held_pattern_keys.iter_mut() {
                    if pattern_data.channel() == ch {
                        *pattern_data
                            .note_data
                            .expression
                            .value_mut(crate::ExprType::Pressure) = Some(val);
                    }
                }
            }
            NoteEvent::MidiCC { channel, cc, value, .. } => {
                let ch = *channel;
                let (kind, mapped) = match *cc {
                    cc::MODULATION_MSB => (crate::ExprType::Vibrato, *value),
                    cc::EXPRESSION_CONTROLLER_MSB => (crate::ExprType::Expression, *value),
                    cc::MAIN_VOLUME_MSB => (crate::ExprType::Volume, *value),
                    cc::PAN_MSB => (crate::ExprType::Pan, (*value) * 2.0 - 1.0),
                    cc::SOUND_CONTROLLER_5 => (crate::ExprType::Brightness, *value),
                    _ => return,
                };
                for (_raw, pattern_data) in self.held_pattern_keys.iter_mut() {
                    if pattern_data.channel() == ch {
                        *pattern_data.note_data.expression.value_mut(kind) = Some(mapped);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn update_chord_expr_from_channel_event(&mut self, event: &PluginNoteEvent<P>) {
        match event {
            NoteEvent::MidiChannelPressure { channel: _, pressure, .. } => {
                let val = *pressure;
                for base in self.chord.iter().copied() {
                    let entry = self
                        .chord_expr
                        .entry(base)
                        .or_insert_with(ExpressionState::default);
                    *entry.value_mut(crate::ExprType::Pressure) = Some(val);
                }
            }
            NoteEvent::MidiCC { cc, value, .. } => {
                let (kind, mapped) = match *cc {
                    cc::MODULATION_MSB => (crate::ExprType::Vibrato, *value),
                    cc::EXPRESSION_CONTROLLER_MSB => (crate::ExprType::Expression, *value),
                    cc::MAIN_VOLUME_MSB => (crate::ExprType::Volume, *value),
                    cc::PAN_MSB => (crate::ExprType::Pan, (*value) * 2.0 - 1.0),
                    cc::SOUND_CONTROLLER_5 => (crate::ExprType::Brightness, *value),
                    _ => return,
                };
                for base in self.chord.iter().copied() {
                    let entry = self
                        .chord_expr
                        .entry(base)
                        .or_insert_with(ExpressionState::default);
                    *entry.value_mut(kind) = Some(mapped);
                }
            }
            _ => {}
        }
    }

    pub fn maybe_emit_expressions(
        &mut self,
        send_events: &mut Vec<PluginNoteEvent<P>>,
        timing: u32,
        mix: f32,
        mirror_cc: bool,
    ) {
        if self.last_expr_emit_sample != 0
            && timing <= self.last_expr_emit_sample + self.expr_tick_interval
        {
            return;
        }

        const KINDS: &[ExprType] = &[
            ExprType::Pressure,
            ExprType::Volume,
            ExprType::Pan,
            ExprType::Tuning,
            ExprType::Vibrato,
            ExprType::Expression,
            ExprType::Brightness,
        ];

        let mut emitted_any = false;
        for (_raw, pd) in self.held_pattern_keys.iter_mut() {
            if let Some(tn) = pd.triggered_note() {
                let voice_id = pd.voice_id();
                let channel = pd.channel();
                // Resolve base chord note by chord index
                let base_note_opt = self.chord.iter().nth(pd.chord_idx() as usize).copied();
                for &kind in KINDS {
                    // Read last values; gracefully fall back to the other side
                    // so the mix never collapses to zero when one side has data.
                    let p_opt = pd.note_data.expression.get(kind);
                    let c_opt = if let Some(base_note) = base_note_opt {
                        self.chord_expr
                            .get(&base_note)
                            .and_then(|state| state.get(kind))
                    } else {
                        None
                    };

                    let (p_sm, c_sm) = match (p_opt, c_opt) {
                        (Some(p), Some(c)) => (p, c),
                        (Some(p), None) => (p, p),
                        (None, Some(c)) => (c, c),
                        (None, None) => {
                            let d = default_expr_value(kind);
                            (d, d)
                        }
                    };
                    let mixed_value = p_sm * (1.0 - mix) + c_sm * mix;
                    // Only emit if any side provided a concrete value
                    if p_opt.is_some() || c_opt.is_some() {
                        let expression_event =
                            new_expr_event::<P>(kind, timing, tn, voice_id, channel, mixed_value);
                        send_events.push(expression_event);
                        if mirror_cc {
                            if let Some(cc_event) = crate::utils::new_cc_from_expr::<P>(
                                kind, timing, channel, mixed_value,
                            ) {
                                send_events.push(cc_event);
                            }
                        }
                        emitted_any = true;
                    }
                }
            }
        }

        if emitted_any {
            self.last_expr_emit_sample = timing;
        }
    }

    /// Finalizes the processing cycle by applying all pending pattern changes.
    ///
    /// This function serves as a wrapper around `apply_pattern_changes` and is meant to be called once
    /// per processing cycle to flush all queued events and update held pattern keys.
    ///
    /// # Parameters
    /// - `send_events`: A mutable vector into which all generated MIDI events are appended.
    /// - `timing`: The sample-accurate timing for this cycle.
    /// - `wrap_threshold`: Determines the modular wrap-around value for chord note mapping.
    /// - `octave_range`: The scaling factor for the octave offset of the triggered note.
    /// - `keyboard_mode`: Specifies how to process the raw note (e.g., ignoring black keys).
    /// - `octave_shift`: Specifies the octave shift to be applied to the triggered note.
    pub fn end_cycle(
        &mut self,
        send_events: &mut Vec<PluginNoteEvent<P>>,
        timing: u32,
        wrap_threshold: u8,
        octave_range: u8,
        keyboard_mode: KeyboardMode,
        octave_shift: i8,
    ) {
        self.apply_pattern_changes(
            send_events,
            timing,
            wrap_threshold,
            octave_range,
            keyboard_mode,
            octave_shift,
        );
    }

    /// Processes a MIDI chord event to update the internal chord state.
    ///
    /// This function inspects the incoming event; if it is a NoteOn event, the note is added to
    /// the chord set, and if it is a NoteOff event, the note is removed.
    ///
    /// # Parameters
    /// - `e`: The incoming MIDI event representing a chord note. Only NoteOn and NoteOff events are processed.
    pub fn process_chord_event(&mut self, e: PluginNoteEvent<P>) {
        match e {
            NoteOn { note, .. } => {
                let _ = self.chord.insert(note);
            }
            NoteOff { note, .. } => {
                let _ = self.chord.remove(&note);
            }
            _ => {}
        }
    }

    /// Queues a pattern event for later processing.
    ///
    /// Depending on whether the event is a NoteOn or a NoteOff, it is added to the appropriate queue:
    /// - NoteOn events are added to `pressed_pattern_keys`.
    /// - NoteOff events are added to `released_pattern_keys`.
    ///
    /// # Parameters
    /// - `e`: The incoming MIDI event representing a pattern note.
    pub fn process_pattern_event(&mut self, e: PluginNoteEvent<P>) {
        match e {
            NoteOn { .. } => self.pressed_pattern_keys.push_back(e),
            NoteOff { .. } => self.released_pattern_keys.push_back(e),
            _ => {}
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::processors::ChordPatternProcessor;
    use crate::utils::KeyboardMode;
    use crate::Patterns;
    use nih_plug::midi::NoteEvent::{NoteOff, NoteOn};
    use nih_plug::midi::PluginNoteEvent;
    use std::collections::BTreeSet;

    /// Tests that processing a chord NoteOn event correctly adds the note to the chord set,
    /// and that processing a NoteOff event removes it.
    #[test]
    fn test_process_chord_event() {
        let mut processor = ChordPatternProcessor::<Patterns>::default();

        // Trigger a NoteOn event for the chord
        processor.process_chord_event(NoteOn {
            note: 60,
            velocity: 1.0,
            voice_id: None,
            timing: 0,
            channel: 16,
        });

        let chord_vec: Vec<u8> = processor.chord.clone().into_iter().collect();
        assert_eq!(chord_vec, [60]);

        // Process corresponding NoteOff event to remove the note
        processor.process_chord_event(NoteOff {
            note: 60,
            velocity: 1.0,
            voice_id: None,
            timing: 0,
            channel: 16,
        });

        let chord_vec: Vec<u8> = processor.chord.clone().into_iter().collect();
        println!("{:?}", chord_vec);
        assert!(chord_vec.is_empty());
    }

    /// Tests that NoteOn events for pattern keys are queued in `pressed_pattern_keys`
    /// and NoteOff events are queued in `released_pattern_keys`.
    #[test]
    fn test_process_pattern_event() {
        let mut processor = ChordPatternProcessor::<Patterns>::default();

        // Process a NoteOn event and verify it is queued correctly.
        processor.process_pattern_event(NoteOn {
            note: 60,
            velocity: 1.0,
            voice_id: None,
            timing: 0,
            channel: 16,
        });

        let pattern_vec: Vec<PluginNoteEvent<Patterns>> =
            processor.pressed_pattern_keys.clone().into_iter().collect();
        assert_eq!(
            pattern_vec,
            [NoteOn {
                note: 60,
                velocity: 1.0,
                voice_id: None,
                timing: 0,
                channel: 16
            }]
        );

        // Process a NoteOff event and verify it is queued correctly.
        processor.process_pattern_event(NoteOff {
            note: 60,
            velocity: 1.0,
            voice_id: None,
            timing: 0,
            channel: 16,
        });

        let pattern_vec: Vec<PluginNoteEvent<Patterns>> = processor
            .released_pattern_keys
            .clone()
            .into_iter()
            .collect();
        assert_eq!(
            pattern_vec,
            [NoteOff {
                note: 60,
                velocity: 1.0,
                voice_id: None,
                timing: 0,
                channel: 16
            }]
        );
    }

    /// Verifies that a full processing cycle correctly transforms pressed pattern notes into
    /// modulated NoteOn events and released pattern notes into modulated NoteOff events based on the
    /// current chord state.
    #[test]
    fn test_end_cycle() {
        let mut processor = ChordPatternProcessor::<Patterns>::default();

        // Set an initial chord state with three notes.
        processor.chord = BTreeSet::from([72, 74, 76]);

        // Process a NoteOn event for a pattern note.
        processor.process_pattern_event(NoteOn {
            note: 60,
            velocity: 1.0,
            voice_id: None,
            timing: 0,
            channel: 16,
        });

        let send_events = &mut vec![];
        processor.end_cycle(send_events, 0, 3, 12, KeyboardMode::AllKeys, 0);

        // Expect that the NoteOn event has been modulated to correspond to the current chord mapping.
        assert_eq!(
            *send_events,
            [NoteOn {
                note: 72,
                velocity: 1.0,
                voice_id: None,
                timing: 0,
                channel: 16,
            }]
        );

        // Process a NoteOff event for the same pattern note.
        processor.process_pattern_event(NoteOff {
            note: 60,
            velocity: 1.0,
            voice_id: None,
            timing: 1,
            channel: 16,
        });

        let send_events = &mut vec![];
        processor.end_cycle(send_events, 1, 3, 12, KeyboardMode::AllKeys, 0);

        // Expect that the NoteOff event is modulated to turn off the previously triggered note.
        assert_eq!(
            *send_events,
            [NoteOff {
                note: 72,
                velocity: 1.0,
                voice_id: None,
                timing: 1,
                channel: 16,
            }]
        );
    }
}
