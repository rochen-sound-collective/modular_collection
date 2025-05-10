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
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
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
#[derive(PartialEq, Debug, Clone)]
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

#[cfg(test)]
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
