use nih_plug::midi::NoteEvent;
use crate::processors::PatternChordData;
use crate::utils;
use nih_plug::prelude::*;

#[derive(Enum, PartialEq)]
pub enum KeyboardMode {
    AllKeys = 0,
    IgnoreBlackKeys = 1,
    //ShiftBlackKeysRight = 2, //Not implemented
    //ShiftBlackKeysLeft = 3
}

pub fn note_to_chord_idx_octave(note: u8, wrap_threshold: u8) -> (u8, i8) {
    (
        //note
        ((note as i32 - 60).rem_euclid(
            wrap_threshold as i32)) as u8,
        //octave
        ((note as i32 - 60).div_euclid(
            wrap_threshold as i32)) as i8,
    )
}

pub fn count_black_keys(note: u8) -> u8 {
    // Calculate the number of octaves between the lowest and the highest note
    let octaves = ((note) / 12) as u8;
    // Return the number of black keys in the octaves
    return (octaves * 5 + [0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 5][(note.rem_euclid(12)) as usize]) as u8;
}

pub fn count_black_keys_from_C3(note: u8) -> i32 {
  // there are 25 black keys from C0 (0) to C3 (60)
  count_black_keys(note) as i32 - 25
}

pub fn is_black_key(note: u8) -> bool {
    match note % 12 {
        1 => true,  // C# / Db
        3 => true,  // D# / Eb
        6 => true,  // F# / Gb
        8 => true,  // G# / Ab
        10 => true, // A# / Bb
        _ => false,
    }
}

pub fn raw_note_apply_keyboard_mode(raw_note: u8, keyboard_mode: &KeyboardMode)-> Option<u8> {
    match keyboard_mode {
        KeyboardMode::IgnoreBlackKeys =>
          match is_black_key(raw_note) {
              true=> None,
              false => u8::try_from(raw_note as i32 - count_black_keys_from_C3(raw_note)).ok()
          }
        KeyboardMode::AllKeys =>  Some(raw_note),
    }
}

pub fn get_chord_data(chord_vec: &Vec<u8>, note_value: u8, wrap_threshold: u8, octave_range: u8) -> PatternChordData {
    let (chord_idx, octave) = utils::note_to_chord_idx_octave(note_value, wrap_threshold);

    //let chord_vec: Vec<u8> = self.chord.iter().cloned().collect();

    let mut chord_data = PatternChordData {
        chord_idx: chord_idx,
        octave: octave,
        triggered_note: None,
    };

    if let Some(note) = chord_vec.get(chord_idx as usize) {
        chord_data.triggered_note = u8::try_from(*note as i8 + octave_range as i8 * octave).ok();
    };

    chord_data
}

pub fn get_note_of_event(note_event: &NoteEvent) -> Option<u8> {
    match note_event {
        NoteEvent::NoteOn { note, .. }
        | NoteEvent::NoteOff { note, .. }
        | NoteEvent::Choke { note, .. }
        | NoteEvent::PolyPressure { note, .. }
        | NoteEvent::PolyVolume { note, .. }
        | NoteEvent::PolyPan { note, .. }
        | NoteEvent::PolyTuning { note, .. }
        | NoteEvent::PolyVibrato { note, .. }
        | NoteEvent::PolyExpression { note, .. }
        | NoteEvent::PolyBrightness { note, .. } => Some(*note),
        _ => None,
    }
}

pub fn set_note_of_event(note_event: &NoteEvent, new_note: u8) -> NoteEvent {
    let mut mut_note_event = note_event.clone();
    match mut_note_event {
        NoteEvent::NoteOn { ref mut note, .. }
        | NoteEvent::NoteOff { ref mut note, .. }
        | NoteEvent::Choke { ref mut note, .. }
        | NoteEvent::PolyPressure { ref mut note, .. }
        | NoteEvent::PolyVolume { ref mut note, .. }
        | NoteEvent::PolyPan { ref mut note, .. }
        | NoteEvent::PolyTuning { ref mut note, .. }
        | NoteEvent::PolyVibrato { ref mut note, .. }
        | NoteEvent::PolyExpression { ref mut note, .. }
        | NoteEvent::PolyBrightness { ref mut note, .. } => *note = new_note,
        _ => (),
    }
    return mut_note_event;
}

pub fn get_velocity_of_event(note_event: &NoteEvent) -> Option<f32> {
    match note_event {
        NoteEvent::NoteOn { velocity, .. }
        | NoteEvent::NoteOff { velocity, .. } => Some(*velocity),
        _ => None,
    }
}

pub fn get_channel_of_event(note_event: &NoteEvent) -> Option<u8> {
    match note_event {
        NoteEvent::NoteOn { channel, .. }
        | NoteEvent::NoteOff { channel, .. }
        | NoteEvent::Choke { channel, .. }
        | NoteEvent::PolyPressure { channel, .. }
        | NoteEvent::PolyVolume { channel, .. }
        | NoteEvent::PolyPan { channel, .. }
        | NoteEvent::PolyTuning { channel, .. }
        | NoteEvent::PolyVibrato { channel, .. }
        | NoteEvent::PolyExpression { channel, .. }
        | NoteEvent::PolyBrightness { channel, .. } => Some(*channel),
        _ => None,
    }
}

pub fn get_voice_id_of_event(note_event: &NoteEvent) -> Option<i32> { // Check if correct events are selected
    match note_event {
        NoteEvent::NoteOn { voice_id, .. }
        | NoteEvent::NoteOff { voice_id, .. }
        | NoteEvent::Choke { voice_id, .. }
        | NoteEvent::PolyPressure { voice_id, .. }
        | NoteEvent::PolyVolume { voice_id, .. }
        | NoteEvent::PolyPan { voice_id, .. }
        | NoteEvent::PolyTuning { voice_id, .. }
        | NoteEvent::PolyVibrato { voice_id, .. }
        | NoteEvent::PolyExpression { voice_id, .. }
        | NoteEvent::PolyBrightness { voice_id, .. } => *voice_id,
        _ => None,
    }
}


#[cfg(test)]
mod tests {
    use crate::utils::{get_channel_of_event, get_chord_data, note_to_chord_idx_octave, is_black_key, count_black_keys_from_C3, raw_note_apply_keyboard_mode, KeyboardMode};
    use nih_plug::midi::NoteEvent;
    use crate::processors::PatternChordData;

    #[test]
    fn test_count_black_keys() {
        assert_eq!(count_black_keys_from_C3(50), -4);
        assert_eq!(count_black_keys_from_C3(51), -3);
        assert_eq!(count_black_keys_from_C3(52), -3);
        assert_eq!(count_black_keys_from_C3(53), -2);
        assert_eq!(count_black_keys_from_C3(54), -2);
        assert_eq!(count_black_keys_from_C3(55), -1);
        assert_eq!(count_black_keys_from_C3(56), -1);
        assert_eq!(count_black_keys_from_C3(57), 0);
        assert_eq!(count_black_keys_from_C3(58), 0);
        assert_eq!(count_black_keys_from_C3(59), 0);
        assert_eq!(count_black_keys_from_C3(60), 0);
        assert_eq!(count_black_keys_from_C3(61), 1);
        assert_eq!(count_black_keys_from_C3(62), 1);
        assert_eq!(count_black_keys_from_C3(63), 2);
        assert_eq!(count_black_keys_from_C3(64), 2);
        assert_eq!(count_black_keys_from_C3(65), 3);
        assert_eq!(count_black_keys_from_C3(66), 3);
        assert_eq!(count_black_keys_from_C3(67), 4);
        assert_eq!(count_black_keys_from_C3(68), 4);
        assert_eq!(count_black_keys_from_C3(69), 5);
        assert_eq!(count_black_keys_from_C3(70), 5);
        assert_eq!(count_black_keys_from_C3(71), 5);
        assert_eq!(count_black_keys_from_C3(72), 5);
        assert_eq!(count_black_keys_from_C3(73), 6);
        assert_eq!(count_black_keys_from_C3(74), 6);
        assert_eq!(count_black_keys_from_C3(75), 7);
    }    

    #[test]
    fn test_get_channel_of_event() {
        let note_channel = get_channel_of_event(&NoteEvent::NoteOn { timing: 0, note: 60, velocity: 1.0, voice_id: None, channel: 1 });
        assert_eq!(note_channel, Some(1));
        let note_channel = get_channel_of_event(&NoteEvent::NoteOff { timing: 0, note: 60, velocity: 1.0, voice_id: None, channel: 16 });
        assert_eq!(note_channel, Some(16));
        let note_channel = get_channel_of_event(&NoteEvent::PolyModulation { timing: 0, poly_modulation_id: 0, voice_id: 0, normalized_offset: 0.0 });
        assert_eq!(note_channel, None);
    }

    #[test]
    fn test_note_to_chord_idx_octave() {
        let (note_index, octave) = note_to_chord_idx_octave(60, 3);
        assert_eq!((note_index, octave), (0, 0));
        let (note_index, octave) = note_to_chord_idx_octave(61, 3);
        assert_eq!((note_index, octave), (1, 0));
        let (note_index, octave) = note_to_chord_idx_octave(62, 3);
        assert_eq!((note_index, octave), (2, 0));
        let (note_index, octave) = note_to_chord_idx_octave(63, 3);
        assert_eq!((note_index, octave), (0, 1));
    }

    #[test]
    fn test_is_black_key(){
        assert!(!is_black_key(60)); // C3 = white
        assert!(is_black_key(61)); // C#3 = black
        assert!(!is_black_key(62)); // D3 = white
        assert!(is_black_key(63)); // D#3 = black
        assert!(!is_black_key(64)); // E3 = white
        assert!(!is_black_key(65)); // F3 = white
        assert!(is_black_key(66)); // F#3 = black
        assert!(!is_black_key(67)); // G3 = white
        assert!(is_black_key(68)); // G#3 = black
        assert!(!is_black_key(69)); // A3 = white
        assert!(is_black_key(70)); // A#3 = black
        assert!(!is_black_key(71)); // B3 = white
        assert!(!is_black_key(72)); // C4 = white
    }

    #[test]
    fn test_raw_note_apply_keyboard_mode(){
        assert_eq!(raw_note_apply_keyboard_mode(60, &KeyboardMode::AllKeys), Some(60));
        assert_eq!(raw_note_apply_keyboard_mode(60, &KeyboardMode::IgnoreBlackKeys), Some(60));
        assert_eq!(raw_note_apply_keyboard_mode(61, &KeyboardMode::AllKeys), Some(61));
        assert_eq!(raw_note_apply_keyboard_mode(61, &KeyboardMode::IgnoreBlackKeys), None);
        assert_eq!(raw_note_apply_keyboard_mode(62, &KeyboardMode::AllKeys), Some(62));
        assert_eq!(raw_note_apply_keyboard_mode(62, &KeyboardMode::IgnoreBlackKeys), Some(61));
        assert_eq!(raw_note_apply_keyboard_mode(63, &KeyboardMode::AllKeys), Some(63));
        assert_eq!(raw_note_apply_keyboard_mode(63, &KeyboardMode::IgnoreBlackKeys), None);
    }

    #[test]
    fn test_get_chord_data() {
        let chord = vec![72, 74, 76];

        // positive octave
        let data = get_chord_data(&chord, 60, 3, 12);
        assert_eq!(PatternChordData {
            octave: 0,
            chord_idx: 0,
            triggered_note: Some(72),
        }, data);

        let data = get_chord_data(&chord, 61, 3, 12);
        assert_eq!(PatternChordData {
            octave: 0,
            chord_idx: 1,
            triggered_note: Some(74),
        }, data);

        let data = get_chord_data(&chord, 62, 3, 12);
        assert_eq!(PatternChordData {
            octave: 0,
            chord_idx: 2,
            triggered_note: Some(76),
        }, data);

        let data = get_chord_data(&chord, 63, 3, 12);
        assert_eq!(PatternChordData {
            octave: 1,
            chord_idx: 0,
            triggered_note: Some(84),
        }, data);

        // invalid chord idx -> no note triggered
        let data = get_chord_data(&chord, 63, 4, 12);
        assert_eq!(PatternChordData {
            octave: 0,
            chord_idx: 3,
            triggered_note: None,
        }, data);

        // negative octave
        let data = get_chord_data(&chord, 59, 3, 12);
        assert_eq!(PatternChordData {
            octave: -1,
            chord_idx: 2,
            triggered_note: Some(64),
        }, data);

        let data = get_chord_data(&chord, 58, 3, 12);
        assert_eq!(PatternChordData {
            octave: -1,
            chord_idx: 1,
            triggered_note: Some(62),
        }, data);

        let data = get_chord_data(&chord, 57, 3, 12);
        assert_eq!(PatternChordData {
            octave: -1,
            chord_idx: 0,
            triggered_note: Some(60),
        }, data);

        // octave range
        let data = get_chord_data(&chord, 61, 1, 24);
        assert_eq!(PatternChordData {
            octave: 1,
            chord_idx: 0,
            triggered_note: Some(96),
        }, data);

        let data = get_chord_data(&chord, 61, 1, 6);
        assert_eq!(PatternChordData {
            octave: 1,
            chord_idx: 0,
            triggered_note: Some(78),
        }, data);

        let data = get_chord_data(&chord, 61, 1, 1);
        assert_eq!(PatternChordData {
            octave: 1,
            chord_idx: 0,
            triggered_note: Some(73),
        }, data);
    }
}
