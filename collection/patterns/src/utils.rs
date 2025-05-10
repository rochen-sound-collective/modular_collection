use nih_plug::midi::PluginNoteEvent;
use crate::processors::PatternChordData;
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

pub fn count_black_keys_from_c3(note: u8) -> i32 {
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
              false => u8::try_from(raw_note as i32 - count_black_keys_from_c3(raw_note)).ok()
          }
        KeyboardMode::AllKeys =>  Some(raw_note),
    }
}

pub fn get_chord_data(chord_vec: &Vec<u8>, note_value: u8, wrap_threshold: u8, octave_range: u8) -> PatternChordData {
    let (chord_idx, octave) = note_to_chord_idx_octave(note_value, wrap_threshold);

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

pub fn get_note_of_event<P: nih_plug::prelude::Plugin>(note_event: &PluginNoteEvent<P>) -> Option<u8> {
    match note_event {
        PluginNoteEvent::<P>::NoteOn { note, .. }
        | PluginNoteEvent::<P>::NoteOff { note, .. }
        | PluginNoteEvent::<P>::Choke { note, .. }
        | PluginNoteEvent::<P>::PolyPressure { note, .. }
        | PluginNoteEvent::<P>::PolyVolume { note, .. }
        | PluginNoteEvent::<P>::PolyPan { note, .. }
        | PluginNoteEvent::<P>::PolyTuning { note, .. }
        | PluginNoteEvent::<P>::PolyVibrato { note, .. }
        | PluginNoteEvent::<P>::PolyExpression { note, .. }
        | PluginNoteEvent::<P>::PolyBrightness { note, .. } => Some(*note),
        _ => None,
    }
}

pub fn set_note_of_event<P: nih_plug::prelude::Plugin>(note_event: &PluginNoteEvent<P>, new_note: u8) -> PluginNoteEvent<P> {
    let mut mut_note_event = note_event.clone();
    match mut_note_event {
        PluginNoteEvent::<P>::NoteOn { ref mut note, .. }
        | PluginNoteEvent::<P>::NoteOff { ref mut note, .. }
        | PluginNoteEvent::<P>::Choke { ref mut note, .. }
        | PluginNoteEvent::<P>::PolyPressure { ref mut note, .. }
        | PluginNoteEvent::<P>::PolyVolume { ref mut note, .. }
        | PluginNoteEvent::<P>::PolyPan { ref mut note, .. }
        | PluginNoteEvent::<P>::PolyTuning { ref mut note, .. }
        | PluginNoteEvent::<P>::PolyVibrato { ref mut note, .. }
        | PluginNoteEvent::<P>::PolyExpression { ref mut note, .. }
        | PluginNoteEvent::<P>::PolyBrightness { ref mut note, .. } => *note = new_note,
        _ => (),
    }
    return mut_note_event;
}

pub fn get_velocity_of_event<P: nih_plug::prelude::Plugin>(note_event: &PluginNoteEvent<P>) -> Option<f32> {
    match note_event {
        PluginNoteEvent::<P>::NoteOn { velocity, .. }
        | PluginNoteEvent::<P>::NoteOff { velocity, .. } => Some(*velocity),
        _ => None,
    }
}

pub fn get_channel_of_event<P: nih_plug::prelude::Plugin>(note_event: &PluginNoteEvent<P>) -> Option<u8> {
    match note_event {
        PluginNoteEvent::<P>::NoteOn { channel, .. }
        | PluginNoteEvent::<P>::NoteOff { channel, .. }
        | PluginNoteEvent::<P>::Choke { channel, .. }
        | PluginNoteEvent::<P>::PolyPressure { channel, .. }
        | PluginNoteEvent::<P>::PolyVolume { channel, .. }
        | PluginNoteEvent::<P>::PolyPan { channel, .. }
        | PluginNoteEvent::<P>::PolyTuning { channel, .. }
        | PluginNoteEvent::<P>::PolyVibrato { channel, .. }
        | PluginNoteEvent::<P>::PolyExpression { channel, .. }
        | PluginNoteEvent::<P>::PolyBrightness { channel, .. } => Some(*channel),
        _ => None,
    }
}

pub fn get_voice_id_of_event<P: nih_plug::prelude::Plugin>(note_event: &PluginNoteEvent<P>) -> Option<i32> { // Check if correct events are selected
    match note_event {
        PluginNoteEvent::<P>::NoteOn { voice_id, .. }
        | PluginNoteEvent::<P>::NoteOff { voice_id, .. }
        | PluginNoteEvent::<P>::Choke { voice_id, .. }
        | PluginNoteEvent::<P>::PolyPressure { voice_id, .. }
        | PluginNoteEvent::<P>::PolyVolume { voice_id, .. }
        | PluginNoteEvent::<P>::PolyPan { voice_id, .. }
        | PluginNoteEvent::<P>::PolyTuning { voice_id, .. }
        | PluginNoteEvent::<P>::PolyVibrato { voice_id, .. }
        | PluginNoteEvent::<P>::PolyExpression { voice_id, .. }
        | PluginNoteEvent::<P>::PolyBrightness { voice_id, .. } => *voice_id,
        _ => None,
    }
}


#[cfg(test)]
mod tests {
    use crate::utils::{get_channel_of_event, get_chord_data, note_to_chord_idx_octave, is_black_key, count_black_keys_from_c3, raw_note_apply_keyboard_mode, KeyboardMode};
    use nih_plug::midi::PluginNoteEvent;
    use crate::Patterns;
    use crate::processors::PatternChordData;

    #[test]
    fn test_count_black_keys() {
        assert_eq!(count_black_keys_from_c3(50), -4);
        assert_eq!(count_black_keys_from_c3(51), -3);
        assert_eq!(count_black_keys_from_c3(52), -3);
        assert_eq!(count_black_keys_from_c3(53), -2);
        assert_eq!(count_black_keys_from_c3(54), -2);
        assert_eq!(count_black_keys_from_c3(55), -1);
        assert_eq!(count_black_keys_from_c3(56), -1);
        assert_eq!(count_black_keys_from_c3(57), 0);
        assert_eq!(count_black_keys_from_c3(58), 0);
        assert_eq!(count_black_keys_from_c3(59), 0);
        assert_eq!(count_black_keys_from_c3(60), 0);
        assert_eq!(count_black_keys_from_c3(61), 1);
        assert_eq!(count_black_keys_from_c3(62), 1);
        assert_eq!(count_black_keys_from_c3(63), 2);
        assert_eq!(count_black_keys_from_c3(64), 2);
        assert_eq!(count_black_keys_from_c3(65), 3);
        assert_eq!(count_black_keys_from_c3(66), 3);
        assert_eq!(count_black_keys_from_c3(67), 4);
        assert_eq!(count_black_keys_from_c3(68), 4);
        assert_eq!(count_black_keys_from_c3(69), 5);
        assert_eq!(count_black_keys_from_c3(70), 5);
        assert_eq!(count_black_keys_from_c3(71), 5);
        assert_eq!(count_black_keys_from_c3(72), 5);
        assert_eq!(count_black_keys_from_c3(73), 6);
        assert_eq!(count_black_keys_from_c3(74), 6);
        assert_eq!(count_black_keys_from_c3(75), 7);
    }    

    #[test]
    fn test_get_channel_of_event() {
        let note_channel = get_channel_of_event::<Patterns>(&PluginNoteEvent::<Patterns>::NoteOn { timing: 0, note: 60, velocity: 1.0, voice_id: None, channel: 1 });
        assert_eq!(note_channel, Some(1));
        let note_channel = get_channel_of_event::<Patterns>(&PluginNoteEvent::<Patterns>::NoteOff { timing: 0, note: 60, velocity: 1.0, voice_id: None, channel: 16 });
        assert_eq!(note_channel, Some(16));
        let note_channel = get_channel_of_event::<Patterns>(&PluginNoteEvent::<Patterns>::PolyModulation { timing: 0, poly_modulation_id: 0, voice_id: 0, normalized_offset: 0.0 });
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
