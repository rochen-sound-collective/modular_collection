use crate::processors::PatternChordData;
use nih_plug::midi::PluginNoteEvent;
use nih_plug::midi::control_change as cc;
use nih_plug::prelude::*;
use std::collections::BTreeSet;

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
        ((note as i32 - 60).rem_euclid(wrap_threshold as i32)) as u8,
        //octave
        ((note as i32 - 60).div_euclid(wrap_threshold as i32)) as i8,
    )
}

pub fn count_black_keys(note: u8) -> u8 {
    // Calculate the number of octaves between the lowest and the highest note
    let octaves = ((note) / 12) as u8;
    // Return the number of black keys in the octaves
    return (octaves * 5 + [0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 5][(note.rem_euclid(12)) as usize])
        as u8;
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

pub fn raw_note_apply_keyboard_mode(raw_note: u8, keyboard_mode: &KeyboardMode) -> Option<u8> {
    match keyboard_mode {
        KeyboardMode::IgnoreBlackKeys => match is_black_key(raw_note) {
            true => None,
            false => u8::try_from(raw_note as i32 - count_black_keys_from_c3(raw_note)).ok(),
        },
        KeyboardMode::AllKeys => Some(raw_note),
    }
}

pub fn get_chord_data(
    chord_vec: &Vec<u8>,
    note_value: u8,
    wrap_threshold: u8,
    octave_range: u8,
    octave_shift: i8,
) -> PatternChordData {
    let (chord_idx, octave) = note_to_chord_idx_octave(note_value, wrap_threshold);

    let mut chord_data = PatternChordData {
        chord_idx: chord_idx,
        octave: octave,
        triggered_note: None,
    };

    if let Some(note) = chord_vec.get(chord_idx as usize) {
        chord_data.triggered_note =
            u8::try_from(*note as i8 + octave_range as i8 * (octave + octave_shift)).ok();
    };

    chord_data
}

pub fn get_note_of_event<P: nih_plug::prelude::Plugin>(
    note_event: &PluginNoteEvent<P>,
) -> Option<u8> {
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

pub fn get_chord_data_from_set(
    chord_set: &BTreeSet<u8>,
    note_value: u8,
    wrap_threshold: u8,
    octave_range: u8,
    octave_shift: i8,
) -> PatternChordData {
    let (chord_idx, octave) = note_to_chord_idx_octave(note_value, wrap_threshold);

    let mut chord_data = PatternChordData {
        chord_idx,
        octave,
        triggered_note: None,
    };

    if let Some(&note) = chord_set.iter().nth(chord_idx as usize) {
        chord_data.triggered_note =
            u8::try_from(note as i8 + octave_range as i8 * (octave + octave_shift)).ok();
    }

    chord_data
}

pub fn set_note_of_event<P: nih_plug::prelude::Plugin>(
    note_event: &PluginNoteEvent<P>,
    new_note: u8,
) -> PluginNoteEvent<P> {
    match note_event {
        NoteEvent::NoteOn {
            timing,
            velocity,
            voice_id,
            channel,
            ..
        } => NoteEvent::NoteOn {
            timing: *timing,
            note: new_note,
            velocity: *velocity,
            voice_id: *voice_id,
            channel: *channel,
        },
        NoteEvent::NoteOff {
            timing,
            velocity,
            voice_id,
            channel,
            ..
        } => NoteEvent::NoteOff {
            timing: *timing,
            note: new_note,
            velocity: *velocity,
            voice_id: *voice_id,
            channel: *channel,
        },
        NoteEvent::Choke {
            timing,
            voice_id,
            channel,
            ..
        } => NoteEvent::Choke {
            timing: *timing,
            note: new_note,
            voice_id: *voice_id,
            channel: *channel,
        },
        NoteEvent::PolyPressure {
            timing,
            voice_id,
            channel,
            pressure,
            ..
        } => NoteEvent::PolyPressure {
            timing: *timing,
            note: new_note,
            voice_id: *voice_id,
            channel: *channel,
            pressure: *pressure,
        },
        NoteEvent::PolyVolume {
            timing,
            voice_id,
            channel,
            gain,
            ..
        } => NoteEvent::PolyVolume {
            timing: *timing,
            note: new_note,
            voice_id: *voice_id,
            channel: *channel,
            gain: *gain,
        },
        NoteEvent::PolyPan {
            timing,
            voice_id,
            channel,
            pan,
            ..
        } => NoteEvent::PolyPan {
            timing: *timing,
            note: new_note,
            voice_id: *voice_id,
            channel: *channel,
            pan: *pan,
        },
        NoteEvent::PolyTuning {
            timing,
            voice_id,
            channel,
            tuning,
            ..
        } => NoteEvent::PolyTuning {
            timing: *timing,
            note: new_note,
            voice_id: *voice_id,
            channel: *channel,
            tuning: *tuning,
        },
        NoteEvent::PolyVibrato {
            timing,
            voice_id,
            channel,
            vibrato,
            ..
        } => NoteEvent::PolyVibrato {
            timing: *timing,
            note: new_note,
            voice_id: *voice_id,
            channel: *channel,
            vibrato: *vibrato,
        },
        NoteEvent::PolyExpression {
            timing,
            voice_id,
            channel,
            expression,
            ..
        } => NoteEvent::PolyExpression {
            timing: *timing,
            note: new_note,
            voice_id: *voice_id,
            channel: *channel,
            expression: *expression,
        },
        NoteEvent::PolyBrightness {
            timing,
            voice_id,
            channel,
            brightness,
            ..
        } => NoteEvent::PolyBrightness {
            timing: *timing,
            note: new_note,
            voice_id: *voice_id,
            channel: *channel,
            brightness: *brightness,
        },
        other => other.clone(),
    }
}

pub fn set_note_voice_channel_of_event<P: nih_plug::prelude::Plugin>(
    note_event: &PluginNoteEvent<P>,
    new_note: u8,
    new_voice_id: Option<i32>,
    new_channel: u8,
) -> PluginNoteEvent<P> {
    match note_event {
        NoteEvent::NoteOn {
            timing,
            velocity,
            ..
        } => NoteEvent::NoteOn {
            timing: *timing,
            note: new_note,
            velocity: *velocity,
            voice_id: new_voice_id,
            channel: new_channel,
        },
        NoteEvent::NoteOff {
            timing,
            velocity,
            ..
        } => NoteEvent::NoteOff {
            timing: *timing,
            note: new_note,
            velocity: *velocity,
            voice_id: new_voice_id,
            channel: new_channel,
        },
        NoteEvent::Choke { timing, .. } => NoteEvent::Choke {
            timing: *timing,
            note: new_note,
            voice_id: new_voice_id,
            channel: new_channel,
        },
        NoteEvent::PolyPressure {
            timing,
            pressure,
            ..
        } => NoteEvent::PolyPressure {
            timing: *timing,
            note: new_note,
            voice_id: new_voice_id,
            channel: new_channel,
            pressure: *pressure,
        },
        NoteEvent::PolyVolume { timing, gain, .. } => NoteEvent::PolyVolume {
            timing: *timing,
            note: new_note,
            voice_id: new_voice_id,
            channel: new_channel,
            gain: *gain,
        },
        NoteEvent::PolyPan { timing, pan, .. } => NoteEvent::PolyPan {
            timing: *timing,
            note: new_note,
            voice_id: new_voice_id,
            channel: new_channel,
            pan: *pan,
        },
        NoteEvent::PolyTuning { timing, tuning, .. } => NoteEvent::PolyTuning {
            timing: *timing,
            note: new_note,
            voice_id: new_voice_id,
            channel: new_channel,
            tuning: *tuning,
        },
        NoteEvent::PolyVibrato { timing, vibrato, .. } => NoteEvent::PolyVibrato {
            timing: *timing,
            note: new_note,
            voice_id: new_voice_id,
            channel: new_channel,
            vibrato: *vibrato,
        },
        NoteEvent::PolyExpression {
            timing,
            expression,
            ..
        } => NoteEvent::PolyExpression {
            timing: *timing,
            note: new_note,
            voice_id: new_voice_id,
            channel: new_channel,
            expression: *expression,
        },
        NoteEvent::PolyBrightness {
            timing,
            brightness,
            ..
        } => NoteEvent::PolyBrightness {
            timing: *timing,
            note: new_note,
            voice_id: new_voice_id,
            channel: new_channel,
            brightness: *brightness,
        },
        other => other.clone(),
    }
}

pub fn try_get_expr_type_value<P: nih_plug::prelude::Plugin>(
    event: &PluginNoteEvent<P>,
) -> Option<(crate::ExprType, f32)> {
    match event {
        NoteEvent::PolyPressure { pressure, .. } => Some((crate::ExprType::Pressure, *pressure)),
        NoteEvent::PolyVolume { gain, .. } => Some((crate::ExprType::Volume, *gain)),
        NoteEvent::PolyPan { pan, .. } => Some((crate::ExprType::Pan, *pan)),
        NoteEvent::PolyTuning { tuning, .. } => Some((crate::ExprType::Tuning, *tuning)),
        NoteEvent::PolyVibrato { vibrato, .. } => Some((crate::ExprType::Vibrato, *vibrato)),
        NoteEvent::PolyExpression { expression, .. } => Some((crate::ExprType::Expression, *expression)),
        NoteEvent::PolyBrightness { brightness, .. } => Some((crate::ExprType::Brightness, *brightness)),
        _ => None,
    }
}

pub fn default_expr_value(kind: crate::ExprType) -> f32 {
    match kind {
        crate::ExprType::Volume => 1.0,
        _ => 0.0,
    }
}

pub fn new_expr_event<P: nih_plug::prelude::Plugin>(
    kind: crate::ExprType,
    timing: u32,
    note: u8,
    voice_id: Option<i32>,
    channel: u8,
    value: f32,
) -> PluginNoteEvent<P> {
    match kind {
        crate::ExprType::Pressure => NoteEvent::PolyPressure {
            timing,
            note,
            voice_id,
            channel,
            pressure: value,
        },
        crate::ExprType::Volume => NoteEvent::PolyVolume {
            timing,
            note,
            voice_id,
            channel,
            gain: value,
        },
        crate::ExprType::Pan => NoteEvent::PolyPan {
            timing,
            note,
            voice_id,
            channel,
            pan: value,
        },
        crate::ExprType::Tuning => NoteEvent::PolyTuning {
            timing,
            note,
            voice_id,
            channel,
            tuning: value,
        },
        crate::ExprType::Vibrato => NoteEvent::PolyVibrato {
            timing,
            note,
            voice_id,
            channel,
            vibrato: value,
        },
        crate::ExprType::Expression => NoteEvent::PolyExpression {
            timing,
            note,
            voice_id,
            channel,
            expression: value,
        },
        crate::ExprType::Brightness => NoteEvent::PolyBrightness {
            timing,
            note,
            voice_id,
            channel,
            brightness: value,
        },
    }
}

pub fn new_cc_from_expr<P: nih_plug::prelude::Plugin>(
    kind: crate::ExprType,
    timing: u32,
    channel: u8,
    value: f32,
) -> Option<PluginNoteEvent<P>> {
    match kind {
        // Map expression to CC11
        crate::ExprType::Expression => Some(NoteEvent::MidiCC {
            timing,
            channel,
            cc: cc::EXPRESSION_CONTROLLER_MSB,
            value,
        }),
        // Map volume to CC7 (gain is a ratio here)
        crate::ExprType::Volume => Some(NoteEvent::MidiCC {
            timing,
            channel,
            cc: cc::MAIN_VOLUME_MSB,
            value,
        }),
        // Map pan to CC10, remap [-1,1] -> [0,1]
        crate::ExprType::Pan => Some(NoteEvent::MidiCC {
            timing,
            channel,
            cc: cc::PAN_MSB,
            value: (value + 1.0) * 0.5,
        }),
        // Map vibrato to CC1 (mod wheel)
        crate::ExprType::Vibrato => Some(NoteEvent::MidiCC {
            timing,
            channel,
            cc: cc::MODULATION_MSB,
            value,
        }),
        // Map pressure to channel pressure
        crate::ExprType::Pressure => Some(NoteEvent::MidiChannelPressure { timing, channel, pressure: value }),
        // Brightness -> CC74
        crate::ExprType::Brightness => Some(NoteEvent::MidiCC {
            timing,
            channel,
            cc: cc::SOUND_CONTROLLER_5,
            value,
        }),
        // No reasonable mapping for tuning
        crate::ExprType::Tuning => None,
    }
}

pub fn set_expr_value<P: nih_plug::prelude::Plugin>(
    event: &PluginNoteEvent<P>,
    value: f32,
) -> PluginNoteEvent<P> {
    match event {
        NoteEvent::PolyPressure {
            timing,
            note,
            voice_id,
            channel,
            ..
        } => NoteEvent::PolyPressure {
            timing: *timing,
            note: *note,
            voice_id: *voice_id,
            channel: *channel,
            pressure: value,
        },
        NoteEvent::PolyVolume {
            timing,
            note,
            voice_id,
            channel,
            ..
        } => NoteEvent::PolyVolume {
            timing: *timing,
            note: *note,
            voice_id: *voice_id,
            channel: *channel,
            gain: value,
        },
        NoteEvent::PolyPan {
            timing,
            note,
            voice_id,
            channel,
            ..
        } => NoteEvent::PolyPan {
            timing: *timing,
            note: *note,
            voice_id: *voice_id,
            channel: *channel,
            pan: value,
        },
        NoteEvent::PolyTuning {
            timing,
            note,
            voice_id,
            channel,
            ..
        } => NoteEvent::PolyTuning {
            timing: *timing,
            note: *note,
            voice_id: *voice_id,
            channel: *channel,
            tuning: value,
        },
        NoteEvent::PolyVibrato {
            timing,
            note,
            voice_id,
            channel,
            ..
        } => NoteEvent::PolyVibrato {
            timing: *timing,
            note: *note,
            voice_id: *voice_id,
            channel: *channel,
            vibrato: value,
        },
        NoteEvent::PolyExpression {
            timing,
            note,
            voice_id,
            channel,
            ..
        } => NoteEvent::PolyExpression {
            timing: *timing,
            note: *note,
            voice_id: *voice_id,
            channel: *channel,
            expression: value,
        },
        NoteEvent::PolyBrightness {
            timing,
            note,
            voice_id,
            channel,
            ..
        } => NoteEvent::PolyBrightness {
            timing: *timing,
            note: *note,
            voice_id: *voice_id,
            channel: *channel,
            brightness: value,
        },
        other => other.clone(),
    }
}

pub fn get_velocity_of_event<P: nih_plug::prelude::Plugin>(
    note_event: &PluginNoteEvent<P>,
) -> Option<f32> {
    match note_event {
        NoteEvent::NoteOn { velocity, .. } | NoteEvent::NoteOff { velocity, .. } => Some(*velocity),
        _ => None,
    }
}

pub fn get_channel_of_event<P: nih_plug::prelude::Plugin>(
    note_event: &PluginNoteEvent<P>,
) -> Option<u8> {
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

pub fn get_voice_id_of_event<P: nih_plug::prelude::Plugin>(
    note_event: &PluginNoteEvent<P>,
) -> Option<i32> {
    // Check if correct events are selected
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
    use crate::processors::PatternChordData;
    use crate::utils::{
        count_black_keys_from_c3, get_channel_of_event, get_chord_data, is_black_key,
        note_to_chord_idx_octave, raw_note_apply_keyboard_mode, KeyboardMode,
    };
    use crate::Patterns;
    use nih_plug::midi::PluginNoteEvent;

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
        let note_channel = get_channel_of_event::<Patterns>(&PluginNoteEvent::<Patterns>::NoteOn {
            timing: 0,
            note: 60,
            velocity: 1.0,
            voice_id: None,
            channel: 1,
        });
        assert_eq!(note_channel, Some(1));
        let note_channel =
            get_channel_of_event::<Patterns>(&PluginNoteEvent::<Patterns>::NoteOff {
                timing: 0,
                note: 60,
                velocity: 1.0,
                voice_id: None,
                channel: 16,
            });
        assert_eq!(note_channel, Some(16));
        let note_channel =
            get_channel_of_event::<Patterns>(&PluginNoteEvent::<Patterns>::PolyModulation {
                timing: 0,
                poly_modulation_id: 0,
                voice_id: 0,
                normalized_offset: 0.0,
            });
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
    fn test_is_black_key() {
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
    fn test_raw_note_apply_keyboard_mode() {
        assert_eq!(
            raw_note_apply_keyboard_mode(60, &KeyboardMode::AllKeys),
            Some(60)
        );
        assert_eq!(
            raw_note_apply_keyboard_mode(60, &KeyboardMode::IgnoreBlackKeys),
            Some(60)
        );
        assert_eq!(
            raw_note_apply_keyboard_mode(61, &KeyboardMode::AllKeys),
            Some(61)
        );
        assert_eq!(
            raw_note_apply_keyboard_mode(61, &KeyboardMode::IgnoreBlackKeys),
            None
        );
        assert_eq!(
            raw_note_apply_keyboard_mode(62, &KeyboardMode::AllKeys),
            Some(62)
        );
        assert_eq!(
            raw_note_apply_keyboard_mode(62, &KeyboardMode::IgnoreBlackKeys),
            Some(61)
        );
        assert_eq!(
            raw_note_apply_keyboard_mode(63, &KeyboardMode::AllKeys),
            Some(63)
        );
        assert_eq!(
            raw_note_apply_keyboard_mode(63, &KeyboardMode::IgnoreBlackKeys),
            None
        );
    }

    #[test]
    fn test_get_chord_data() {
        let chord = vec![72, 74, 76];

        // positive octave
        let data = get_chord_data(&chord, 60, 3, 12, 0);
        assert_eq!(
            PatternChordData {
                octave: 0,
                chord_idx: 0,
                triggered_note: Some(72),
            },
            data
        );

        let data = get_chord_data(&chord, 61, 3, 12, 0);
        assert_eq!(
            PatternChordData {
                octave: 0,
                chord_idx: 1,
                triggered_note: Some(74),
            },
            data
        );

        let data = get_chord_data(&chord, 62, 3, 12, 0);
        assert_eq!(
            PatternChordData {
                octave: 0,
                chord_idx: 2,
                triggered_note: Some(76),
            },
            data
        );

        let data = get_chord_data(&chord, 63, 3, 12, 0);
        assert_eq!(
            PatternChordData {
                octave: 1,
                chord_idx: 0,
                triggered_note: Some(84),
            },
            data
        );

        // invalid chord idx -> no note triggered
        let data = get_chord_data(&chord, 63, 4, 12, 0);
        assert_eq!(
            PatternChordData {
                octave: 0,
                chord_idx: 3,
                triggered_note: None,
            },
            data
        );

        // negative octave
        let data = get_chord_data(&chord, 59, 3, 12, 0);
        assert_eq!(
            PatternChordData {
                octave: -1,
                chord_idx: 2,
                triggered_note: Some(64),
            },
            data
        );

        let data = get_chord_data(&chord, 58, 3, 12, 0);
        assert_eq!(
            PatternChordData {
                octave: -1,
                chord_idx: 1,
                triggered_note: Some(62),
            },
            data
        );

        let data = get_chord_data(&chord, 57, 3, 12, 0);
        assert_eq!(
            PatternChordData {
                octave: -1,
                chord_idx: 0,
                triggered_note: Some(60),
            },
            data
        );

        // octave range
        let data = get_chord_data(&chord, 61, 1, 24, 0);
        assert_eq!(
            PatternChordData {
                octave: 1,
                chord_idx: 0,
                triggered_note: Some(96),
            },
            data
        );

        let data = get_chord_data(&chord, 61, 1, 6, 0);
        assert_eq!(
            PatternChordData {
                octave: 1,
                chord_idx: 0,
                triggered_note: Some(78),
            },
            data
        );

        let data = get_chord_data(&chord, 61, 1, 1, 0);
        assert_eq!(
            PatternChordData {
                octave: 1,
                chord_idx: 0,
                triggered_note: Some(73),
            },
            data
        );

        // octave shift
        let data = get_chord_data(&chord, 60, 3, 12, 12);
        assert_eq!(
            PatternChordData {
                octave: 0,
                chord_idx: 0,
                triggered_note: Some(84),
            },
            data
        );

        let data = get_chord_data(&chord, 60, 3, 12, -12);
        assert_eq!(
            PatternChordData {
                octave: 0,
                chord_idx: 0,
                triggered_note: Some(60),
            },
            data
        );
    }
}
