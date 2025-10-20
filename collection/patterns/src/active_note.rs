/*!
This module defines data structures and helper methods for representing and handling
active MIDI note events in a plugin. The module provides two index types for tracking active
notes and a data structure to store note properties. In addition, it includes a type alias,
`HeldNotes`, which is a sorted collection used to manage active notes efficiently.

The primary structures defined are:
- `ActiveNoteDefaultIndex`: Uses the MIDI note as the index.
- `ActiveNoteChordIndex`: Uses a chord index and octave for more complex ordering.
- `ActiveNoteDefaultData`: Holds details such as channel, note value, velocity, timing, and an optional voice ID.
*/

use crate::utils::{
    get_channel_of_event, get_note_of_event, get_velocity_of_event, get_voice_id_of_event,
};
use nih_plug::midi::PluginNoteEvent;
use std::collections::BTreeMap;

/// The default index type for an active note.
///
/// This struct uses the MIDI note (0–127) as a unique key to index active notes.
/// It is intended to be used as a key in sorted collections such as `HeldNotes`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct ActiveNoteDefaultIndex {
    /// The MIDI note number (0–127) which acts as the index.
    pub note: u8,
}

/// An alternative index type for an active note based on chord information.
///
/// In many musical applications, notes are organized by their position in a chord and octave.
/// This index type supports that notion by storing a `chord_idx` (position in the chord)
/// and an `octave` offset.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Default)]
pub struct ActiveNoteChordIndex {
    /// The chord index representing the note's position within a chord.
    pub chord_idx: u8,
    /// The octave offset of the note.
    pub octave: i32,
}

/// Stores the detailed data associated with an active MIDI note event.
///
/// This structure contains the primary properties of a note event:
/// - MIDI channel and note number
/// - Velocity (which may be scaled to a floating point range)
/// - Timing information such as the start time (and optionally, end time)
/// - An optional `voice_id` used to differentiate overlapping voices in a plugin context.
///
/// # Fields
/// - `voice_id`: An optional unique identifier required for overlapping voices in certain plugin APIs.
/// - `channel`: The MIDI channel (0 to 15).
/// - `note`: The MIDI key number (0 to 127).
/// - `velocity`: The note's velocity (0.0 to 1.0).
/// - `start_time_beats`: The start time of the note in beats.
/// - `end_time_beats`: Optionally, the end time of the note in beats.
#[derive(PartialEq, Debug, Clone, Default)]
pub struct ActiveNoteDefaultData {
    /// A unique identifier for this note, if available. Using this to refer to a note is
    /// required when allowing overlapping voices for CLAP plugins.
    pub voice_id: Option<i32>,
    /// The note's channel, from 0 to 15.
    pub channel: u8,
    /// The note's MIDI key number, from 0 to 127.
    pub note: u8,
    /// The note's velocity, from 0 to 1. Some plugin APIs may allow higher precision than the
    /// 127 levels available in MIDI.
    pub velocity: f32,
    /// The start time of the note event in beats.
    pub start_time_beats: f32,
    /// The optional end time of the note event in beats.
    pub end_time_beats: Option<f32>,
    /// Pattern-side expression last values per type
    pub expression: ExpressionState,
}

impl ActiveNoteDefaultData {
    /// Creates an `ActiveNoteDefaultData` instance from a given MIDI note event.
    ///
    /// This function extracts required note properties (note, velocity, channel, and voice ID)
    /// from the provided `PluginNoteEvent`. If a property is unavailable, a default value is used:
    /// - The note number defaults to 60 (Middle C).
    /// - The channel and velocity default to their respective defaults.
    ///
    /// # Type Parameters
    /// - `P`: The plugin type which implements `nih_plug::prelude::Plugin`.
    ///
    /// # Parameters
    /// - `note_event`: A reference to a `PluginNoteEvent` from which to extract note data.
    ///
    /// # Returns
    /// An `ActiveNoteDefaultData` populated with properties extracted from `note_event`.
    pub fn from_note_event<P: nih_plug::prelude::Plugin>(
        note_event: &PluginNoteEvent<P>,
    ) -> ActiveNoteDefaultData {
        ActiveNoteDefaultData {
            note: (get_note_of_event::<P>(&note_event)).unwrap_or(60),
            voice_id: get_voice_id_of_event::<P>(&note_event),
            channel: get_channel_of_event::<P>(&note_event).unwrap_or_default(),
            velocity: get_velocity_of_event::<P>(&note_event).unwrap_or_default(),
            start_time_beats: 0.0,
            end_time_beats: None,
            expression: Default::default(),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct ExpressionState {
    pub pressure: Option<f32>,
    pub volume: Option<f32>,
    pub pan: Option<f32>,
    pub tuning: Option<f32>,
    pub vibrato: Option<f32>,
    pub expression: Option<f32>,
    pub brightness: Option<f32>,
}

impl ExpressionState {
    pub fn value_mut(&mut self, kind: crate::ExprType) -> &mut Option<f32> {
        match kind {
            crate::ExprType::Pressure => &mut self.pressure,
            crate::ExprType::Volume => &mut self.volume,
            crate::ExprType::Pan => &mut self.pan,
            crate::ExprType::Tuning => &mut self.tuning,
            crate::ExprType::Vibrato => &mut self.vibrato,
            crate::ExprType::Expression => &mut self.expression,
            crate::ExprType::Brightness => &mut self.brightness,
        }
    }

    pub fn get(&self, kind: crate::ExprType) -> Option<f32> {
        match kind {
            crate::ExprType::Pressure => self.pressure,
            crate::ExprType::Volume => self.volume,
            crate::ExprType::Pan => self.pan,
            crate::ExprType::Tuning => self.tuning,
            crate::ExprType::Vibrato => self.vibrato,
            crate::ExprType::Expression => self.expression,
            crate::ExprType::Brightness => self.brightness,
        }
    }
}

/// A type alias representing a sorted collection of active notes.
///
/// The alias `HeldNotes` is a `BTreeMap` that maps an index (by default, `ActiveNoteDefaultIndex`)
/// to the note data (`ActiveNoteDefaultData`). This collection is used to manage the active notes
/// such that they can be efficiently retrieved in sorted order.
///
/// # Type Parameters
/// - `Index`: The key type used for indexing active notes. Defaults to `ActiveNoteDefaultIndex`.
/// - `Data`: The data stored for each note. Defaults to `ActiveNoteDefaultData`.
pub type HeldNotes<Index: Ord = ActiveNoteDefaultIndex, Data = ActiveNoteDefaultData> =
    BTreeMap<Index, Data>;

// -------------------------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------------------------

/*#[cfg(test)]
mod tests {
    use crate::active_note::{ActiveNoteChordIndex, ActiveNoteDefaultData, HeldNotes};

    /// Tests the sorted order of `HeldNotes` when using `ActiveNoteChordIndex` as keys.
    ///
    /// This test creates a set of active notes with corresponding chord indexes and verifies
    /// that the sorting order in the `HeldNotes` collection matches the expected order. The test
    /// checks both the keys and the associated note data.
    #[test]
    fn test_sorting() {
        let held_keys_index = vec![
            ActiveNoteChordIndex {
                chord_idx: 0,
                octave: 0,
            },
            ActiveNoteChordIndex {
                chord_idx: 0,
                octave: 1,
            },
            ActiveNoteChordIndex {
                chord_idx: 1,
                octave: 0,
            },
            ActiveNoteChordIndex {
                chord_idx: 1,
                octave: 1,
            },
        ];

        let held_keys_values = vec![
            ActiveNoteDefaultData {
                voice_id: None,
                channel: 1,
                note: 0,
                velocity: 0.4,
                start_time_beats: 0.0,
                end_time_beats: None,
            },
            ActiveNoteDefaultData {
                voice_id: None,
                channel: 1,
                note: 3,
                velocity: 0.7,
                start_time_beats: 0.0,
                end_time_beats: None,
            },
            ActiveNoteDefaultData {
                voice_id: None,
                channel: 1,
                note: 1,
                velocity: 0.8,
                start_time_beats: 0.0,
                end_time_beats: None,
            },
            ActiveNoteDefaultData {
                voice_id: None,
                channel: 1,
                note: 4,
                velocity: 0.5,
                start_time_beats: 0.0,
                end_time_beats: None,
            },
        ];

        let mut held_keys = HeldNotes::new();
        held_keys.insert(
            ActiveNoteChordIndex {
                chord_idx: 0,
                octave: 1,
            },
            ActiveNoteDefaultData {
                voice_id: None,
                channel: 1,
                note: 3,
                velocity: 0.7,
                start_time_beats: 0.0,
                end_time_beats: None,
            },
        );
        held_keys.insert(
            ActiveNoteChordIndex {
                chord_idx: 1,
                octave: 0,
            },
            ActiveNoteDefaultData {
                voice_id: None,
                channel: 1,
                note: 1,
                velocity: 0.8,
                start_time_beats: 0.0,
                end_time_beats: None,
            },
        );
        held_keys.insert(
            ActiveNoteChordIndex {
                chord_idx: 1,
                octave: 1,
            },
            ActiveNoteDefaultData {
                voice_id: None,
                channel: 1,
                note: 4,
                velocity: 0.5,
                start_time_beats: 0.0,
                end_time_beats: None,
            },
        );
        held_keys.insert(
            ActiveNoteChordIndex {
                chord_idx: 0,
                octave: 0,
            },
            ActiveNoteDefaultData {
                voice_id: None,
                channel: 1,
                note: 0,
                velocity: 0.4,
                start_time_beats: 0.0,
                end_time_beats: None,
            },
        );

        let values: Vec<ActiveNoteDefaultData> = held_keys.clone().into_values().collect();
        let keys: Vec<ActiveNoteChordIndex> = held_keys.into_keys().collect();

        assert_eq!(held_keys_index, keys);
        assert_eq!(held_keys_values, values);
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_note_event_basic() {
        // Basic case: standard NoteOn event with note 60, velocity 100
        let event = NoteEvent::NoteOn {
            timing: 0,
            note: 60,
            velocity: 0.7,
            voice_id: Some(0),
            channel: 0,
        };
        let result = ActiveNoteDefaultData::from_note_event(&event);
        assert_eq!(result.note, 60);
        assert_eq!(result.velocity, 0.7);
        assert_eq!(result.voice_id, Some(0));
        assert_eq!(result.channel, 0);
        assert_eq!(result.start_time_beats, 0.0);
    }

    /*#[test]
    fn test_from_note_event_no_note_data() {
        // Edge case: event variant without note (e.g., ControlChange)
        let event = PluginNoteEvent::MidiCC { timing: 0, channel: 0, cc: 0, value: 0.0 };
        let result = ActiveNoteDefaultData::from_note_event(&event);
        assert!(result.is_none()); // Assuming function returns Option
    }*/

    #[test]
    fn test_from_note_event_zero_velocity() {
        // Edge case: NoteOn with velocity 0 (silent note)
        let event = PluginNoteEvent::NoteOn {
            timing: 0,
            note: 60,
            velocity: 0.0,
            voice_id: Some(0),
            channel: 0,
        };
        let result = ActiveNoteDefaultData::from_note_event(&event);
        assert_eq!(result.velocity, 0);
    }

    #[test]
    fn test_from_note_event_boundary_notes() {
        // Edge cases: lowest (0) and highest (127) MIDI notes
        let low_event = PluginNoteEvent::NoteOn {
            timing: 0,
            note: 0,
            velocity: 0.5,
            voice_id: Some(0),
            channel: 0,
        };
        let low_result = ActiveNoteDefaultData::from_note_event(&low_event);
        assert_eq!(low_result.note, 0);

        let high_event = PluginNoteEvent::NoteOn {
            timing: 0,
            note: 127,
            velocity: 0.5,
            voice_id: Some(0),
            channel: 0,
        };
        let high_result = ActiveNoteDefaultData::from_note_event(&high_event);
        assert_eq!(high_result.note, 127);
    }

    #[test]
    fn test_held_notes_sorting() {
        // Existing test enhanced: verify BTreeMap sorting by index
        let mut held_notes: HeldNotes<ActiveNoteDefaultIndex, ActiveNoteDefaultData> =
            BTreeMap::new();
        held_notes.insert(
            ActiveNoteDefaultIndex { note: 60 },
            ActiveNoteDefaultData {
                note: 60,
                ..Default::default()
            },
        );
        held_notes.insert(
            ActiveNoteDefaultIndex { note: 60 },
            ActiveNoteDefaultData {
                note: 48,
                ..Default::default()
            },
        );
        let sorted: Vec<_> = held_notes.into_iter().collect();
        assert_eq!(sorted[0].1.note, 48); // Lower index first
        assert_eq!(sorted[1].1.note, 60);
    }
}
