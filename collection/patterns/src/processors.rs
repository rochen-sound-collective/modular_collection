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

use crate::active_note::ActiveNoteDefaultData;
use crate::utils::{get_chord_data, get_note_of_event, raw_note_apply_keyboard_mode, KeyboardMode};
use nih_plug::midi::NoteEvent::{NoteOff, NoteOn};
use nih_plug::midi::PluginNoteEvent;
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
    pub fn note_on<P: nih_plug::prelude::Plugin>(
        &self,
        timing: u32,
    ) -> Option<PluginNoteEvent<P>> {
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
                let chord_data = get_chord_data(
                    &self.chord.iter().cloned().collect(),
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
                if let Some(active_note) = self.held_pattern_keys.remove(&raw_note) {
                    if let Some(modulated_event) = active_note.note_off::<P>(note_event.timing()) {
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
        for (idx, e) in self.held_pattern_keys.iter_mut() {
            let chord_data = get_chord_data(
                &self.chord.iter().cloned().collect(),
                *idx,
                wrap_threshold,
                octave_range,
                octave_shift,
            );
            if e.chord_data != chord_data {
                if let Some(modulated_event) = e.note_off::<P>(timing) {
                    send_events.push(modulated_event);
                }
                e.chord_data = chord_data;
                if let Some(modulated_event) = e.note_on::<P>(timing) {
                    send_events.push(modulated_event);
                }
            }
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
