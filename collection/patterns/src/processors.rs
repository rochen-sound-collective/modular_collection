use std::collections::{BTreeMap, BTreeSet, VecDeque};
use nih_plug::midi::NoteEvent::{NoteOn, NoteOff};
use nih_plug::prelude::*;
use crate::active_note::ActiveNoteDefaultData;

use crate::utils::{get_note_of_event, get_chord_data, KeyboardMode, raw_note_apply_keyboard_mode};


#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Default)]
pub struct PatternChordData {
    pub chord_idx: u8,
    pub octave: i8,
    pub triggered_note: Option<u8>,
}

pub struct PatternData {
    chord_data: PatternChordData,
    note_data: ActiveNoteDefaultData,
}

impl PatternData {
    pub fn note_on<P: nih_plug::prelude::Plugin>(&self, timing: u32) -> Option<PluginNoteEvent<P>> {
        if let Some(modulated_note) = self.chord_data.triggered_note {
            Some(NoteOn {
                note: modulated_note,
                channel: self.note_data.channel,
                velocity: self.note_data.velocity,
                voice_id: self.note_data.voice_id,
                timing: timing,
            })
        } else { None }
    }

    pub fn note_off<P: nih_plug::prelude::Plugin>(&self, timing: u32) -> Option<PluginNoteEvent<P>> {
        if let Some(modulated_note) = self.chord_data.triggered_note {
            Some(NoteOff {
                note: modulated_note,
                channel: self.note_data.channel,
                velocity: self.note_data.velocity,
                voice_id: self.note_data.voice_id,
                timing: timing,
            })
        } else { None }
    }
}

#[derive(Default)]
pub struct ChordPatternProcessor<P: nih_plug::prelude::Plugin> {
    pub pressed_pattern_keys: VecDeque<PluginNoteEvent<P>>,
    pub released_pattern_keys: VecDeque<PluginNoteEvent<P>>,

    pub held_pattern_keys: BTreeMap<u8, PatternData>,
    pub chord: BTreeSet<u8>,
}

impl <P: nih_plug::prelude::Plugin> ChordPatternProcessor<P> {
    /*
    for e in events {
        process_note_event(e)
    } */

    fn apply_pattern_changes(&mut self, send_events: &mut Vec<PluginNoteEvent<P>>, timing: u32,
                             wrap_threshold: u8, octave_range: u8, keyboard_mode: KeyboardMode) {
        // released keys
        while let Some(note_event) = self.released_pattern_keys.pop_back() {
            if let Some(raw_note) = get_note_of_event::<P>(&note_event)
                                        .and_then(|note| raw_note_apply_keyboard_mode(note, &keyboard_mode)){
                if let Some(active_note) = self.held_pattern_keys.remove(&raw_note) {
                    if let Some(modulated_event) = active_note.note_off::<P>(note_event.timing()) {
                        send_events.push(modulated_event);
                    }
                }
            }
        }

        // changes in chord
        for (idx, e) in self.held_pattern_keys.iter_mut() {
            let chord_data = get_chord_data(&self.chord.iter().cloned().collect(), *idx, wrap_threshold, octave_range);
            if e.chord_data != chord_data { // chord changed
                // release notes if triggered
                if let Some(modulated_event) = e.note_off::<P>(timing) {
                    send_events.push(modulated_event);
                }

                e.chord_data = chord_data; //update chord data

                // press note if triggered note is valid
                if let Some(modulated_event) = e.note_on::<P>(timing) {
                    send_events.push(modulated_event);
                }
            }
        }

        // pressed keys
        while let Some(note_event) = self.pressed_pattern_keys.pop_back() {
            if let Some(raw_note) = get_note_of_event::<P>(&note_event)
                                        .and_then(|note| raw_note_apply_keyboard_mode(note, &keyboard_mode)){
                let chord_data = get_chord_data(&self.chord.iter().cloned().collect(), raw_note, wrap_threshold, octave_range);

                let active_note = PatternData {
                  chord_data: chord_data,
                  note_data: ActiveNoteDefaultData::from_note_event::<P>(&note_event),
                };

                if let Some(modulated_event) = active_note.note_on::<P>(note_event.timing()) {
                  send_events.push(modulated_event);
                }
                self.held_pattern_keys.insert(raw_note, active_note);
          }
        }
    }

    //----------------------------

    pub fn end_cycle(&mut self, send_events: &mut Vec<PluginNoteEvent<P>>, timing: u32, wrap_threshold: u8,
                     octave_range: u8, keyboard_mode: KeyboardMode) {
        self.apply_pattern_changes(send_events, timing, wrap_threshold, octave_range, keyboard_mode);
    }

    //----------------------------

    /*
    fn process_note_event(&mut self, e: PluginNoteEvent) {
        if e.channel == settings.chord_channel {
            self.process_chord_event(e)
        } else {
            self.process_pattern_event(e);
        }
    }*/
    //----------------------------

    pub fn process_chord_event(&mut self, e: PluginNoteEvent<P>) {
        match e {
            NoteOn{note, ..} => {let _ = self.chord.insert(note);},
            NoteOff{note, ..} => {let _ = self.chord.remove(&note);},
            _ => {}
        }
    }

    //----------------------------

    pub fn process_pattern_event(&mut self, e: PluginNoteEvent<P>) {
        match e {
            NoteOn{..} => self.pressed_pattern_keys.push_back(e),
            NoteOff{..} => self.released_pattern_keys.push_back(e),
            _ => {}
        }
    }
}


// Tests
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use nih_plug::midi::PluginNoteEvent;
    use nih_plug::midi::NoteEvent::{NoteOn, NoteOff};
    use crate::Patterns;
    use crate::processors::{ChordPatternProcessor};
    use crate::utils::KeyboardMode;

    #[test]
    fn test_process_chord_event() {
        // note on
        let mut processor = ChordPatternProcessor::<Patterns>::default();

        processor.process_chord_event(NoteOn {
            note:60,
            velocity: 1.0,
            voice_id: None,
            timing: 0,
            channel: 16
        });

        let chord_vec: Vec<u8> = processor.chord.clone().into_iter().collect();
        assert_eq!(chord_vec, [60]);

        processor.process_chord_event(NoteOff {
            note:60,
            velocity: 1.0,
            voice_id: None,
            timing: 0,
            channel: 16
        });

        let chord_vec: Vec<u8> = processor.chord.clone().into_iter().collect();
        println!("{:?}", chord_vec);
        assert!(chord_vec.is_empty());
    }

    #[test]
    fn test_process_pattern_event() {
        // note on
        let mut processor = ChordPatternProcessor::<Patterns>::default();

        processor.process_pattern_event(NoteOn {
            note:60,
            velocity: 1.0,
            voice_id: None,
            timing: 0,
            channel: 16
        });

        let pattern_vec: Vec<PluginNoteEvent::<Patterns>> = processor.pressed_pattern_keys.clone().into_iter().collect();
        assert_eq!(pattern_vec,
           [NoteOn {
            note:60,
            velocity: 1.0,
            voice_id: None,
            timing: 0,
            channel: 16
        }]);

        processor.process_pattern_event(NoteOff {
            note:60,
            velocity: 1.0,
            voice_id: None,
            timing: 0,
            channel: 16
        });

        let pattern_vec: Vec<PluginNoteEvent::<Patterns>> = processor.released_pattern_keys.clone().into_iter().collect();
        assert_eq!(pattern_vec,
           [NoteOff {
            note:60,
            velocity: 1.0,
            voice_id: None,
            timing: 0,
            channel: 16
        }]);
    }

    #[test]
    fn test_end_cycle() {
        // note on
        let mut processor = ChordPatternProcessor::<Patterns>::default();

        processor.chord = BTreeSet::from([72, 74, 76]);

        // press pattern note
        //---------------------------------->
        processor.process_pattern_event(NoteOn {
            note:60,
            velocity: 1.0,
            voice_id: None,
            timing: 0,
            channel: 16
        });

        let send_events = &mut vec![];
        processor.end_cycle(send_events, 0, 3, 12, KeyboardMode::AllKeys);

        assert_eq!(*send_events, [
            NoteOn {
                note:72,
                velocity: 1.0,
                voice_id: None,
                timing: 0,
                channel: 16
        }]);
        //<----------------------------------

        // held pattern keys
        //---------------------------------->

        //<----------------------------------

        // release pattern note
        //---------------------------------->
        processor.process_pattern_event(NoteOff {
            note:60,
            velocity: 1.0,
            voice_id: None,
            timing: 1,
            channel: 16
        });

        let send_events = &mut vec![];
        processor.end_cycle(send_events, 1, 3, 12, KeyboardMode::AllKeys);

        assert_eq!(*send_events, [
            NoteOff {
                note:72,
                velocity: 1.0,
                voice_id: None,
                timing: 1,
                channel: 16
        }]);
        //<----------------------------------

    }
}