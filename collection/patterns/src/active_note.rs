use std::collections::{BTreeMap};
use nih_plug::midi::NoteEvent;
use crate::utils::{get_channel_of_event, get_note_of_event, get_velocity_of_event, get_voice_id_of_event};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct ActiveNoteDefaultIndex {
    pub note: u8,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct ActiveNoteChordIndex {
    pub chord_idx: u8,
    pub octave: i32,
}

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
}

impl ActiveNoteDefaultData {
    pub fn from_note_event(note_event: &NoteEvent)->ActiveNoteDefaultData{
        ActiveNoteDefaultData{
            note: get_note_of_event(&note_event).unwrap_or(60),
            voice_id: get_voice_id_of_event(&note_event),
            channel: get_channel_of_event(&note_event).unwrap_or_default(),
            velocity: get_velocity_of_event(&note_event).unwrap_or_default(),
        }
    }
}

pub type HeldNotes <Index: Ord = ActiveNoteDefaultIndex, Data = ActiveNoteDefaultData> = BTreeMap<Index, Data>;


// Tests
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::active_note::{ActiveNoteDefaultData, ActiveNoteChordIndex, HeldNotes};

    #[test]
    fn test_sorting() {
        let held_keys_index = vec![
            ActiveNoteChordIndex {chord_idx:0,octave:0},
            ActiveNoteChordIndex {chord_idx:0,octave:1},
            ActiveNoteChordIndex {chord_idx:1,octave:0},
            ActiveNoteChordIndex {chord_idx:1,octave:1},
        ];

        let held_keys_values = vec![
            ActiveNoteDefaultData {
                 voice_id: None,
                 channel: 1,
                 note: 0,
                 velocity: 0.4,
            },
            ActiveNoteDefaultData {
                 voice_id: None,
                 channel: 1,
                 note: 3,
                 velocity: 0.7,
            },
            ActiveNoteDefaultData {
                 voice_id: None,
                 channel: 1,
                 note: 1,
                 velocity: 0.8,
            },
            ActiveNoteDefaultData {
                 voice_id: None,
                 channel: 1,
                 note: 4,
                 velocity: 0.5,
            }
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
            }
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
            }
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
            }
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
            }
        );


        let values: Vec<ActiveNoteDefaultData> = held_keys.clone().into_values().collect();
        let keys: Vec<ActiveNoteChordIndex> = held_keys.into_keys().collect();

        assert_eq!(held_keys_index, keys);
        assert_eq!(held_keys_values, values);
    }
}
