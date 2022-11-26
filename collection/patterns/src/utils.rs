use nih_plug::midi::NoteEvent;
use crate::processors::PatternChordData;
use crate::utils;

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

pub fn get_chord_data(chord_vec: &Vec<u8>, note_value: u8, wrap_threshold: u8, octave_range: u8) -> PatternChordData {
    let (chord_idx, octave) = utils::note_to_chord_idx_octave(note_value, wrap_threshold);

    //let chord_vec: Vec<u8> = self.chord.iter().cloned().collect();

    let mut chord_data = PatternChordData {
        chord_idx: chord_idx,
        octave: octave,
        triggered_note: None,
    };

    if let Some(note) = chord_vec.get(chord_idx as usize) {
        chord_data.triggered_note = Some((*note as i8 + octave_range as i8 * octave) as u8);
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
    use crate::utils::{get_channel_of_event, get_chord_data, note_to_chord_idx_octave};
    use nih_plug::midi::NoteEvent;
    use crate::processors::PatternChordData;

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