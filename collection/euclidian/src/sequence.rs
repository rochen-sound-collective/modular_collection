#[derive(Clone, PartialEq, Debug)]
pub struct SeqNoteEvent {
    pub sample_pos: i64,
    pub note_data: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Sequence {
    pub note_events: Vec<SeqNoteEvent>,
    pub sequence_length: i64,
}

impl Default for Sequence {
    fn default() -> Self {
        Self {
            note_events: vec![],
            sequence_length: 1,
        }

    }
}

impl Sequence {
    pub fn add_note_event(&mut self, event: SeqNoteEvent) {
        // Add the new note event to the vector
        self.note_events.push(event);
    }

    pub fn beats_to_samples(&self, beat_position: f64, tempo: f64, sample_rate: f32) -> i64 {
        (beat_position / tempo.max(0.00001) * 60.0 * sample_rate as f64).round() as i64
    }

    pub fn get_wrapped_sample_position(&self, sample_position: i64) -> i64 {
        let wrapped_sample_position = sample_position % self.sequence_length.max(1);

        if wrapped_sample_position < 0 {
            self.sequence_length + wrapped_sample_position
        } else {
            wrapped_sample_position
        }
    }

    pub fn get_note_events_at_sample(&self, wrapped_sample_position: i64) -> Vec<SeqNoteEvent> {
        self.note_events
            .iter()
            .filter(|event| event.sample_pos == wrapped_sample_position)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::sequence::{Sequence, SeqNoteEvent};

    #[test]
    fn sequence_beats_to_samples() {
            let sequence = Sequence::default();

            assert_eq!(sequence.beats_to_samples(8.0, 120.0, 44_100.0),
                       (8.0 / 120.0 * 60.0 * 44_100.0 as f64).round() as i64);
            assert_eq!(sequence.beats_to_samples(1.0, 120.0, 44_100.0),
                       (1.0 / 120.0 * 60.0 * 44_100.0 as f64).round() as i64);

            assert_eq!(sequence.beats_to_samples(1.0, 0.0, 0.0), // no crash on 0
                       (0) as i64);
    }

    #[test]
    fn sequence_add() {
            let mut sequence = Sequence::default();

            sequence.sequence_length = sequence.beats_to_samples(8.0, 120.0, 44_100.0);

            let note_length = sequence.beats_to_samples(1.0, 120.0, 44_100.0);

            sequence.add_note_event(SeqNoteEvent {
                sample_pos: 0,
                note_data: true,
            });
            sequence.add_note_event(SeqNoteEvent {
                sample_pos: note_length,
                note_data: false,
            });
    }

    #[test]
    fn sequence_get_wrapped_sample_position() {
            let mut sequence = Sequence::default();

            sequence.sequence_length = 22_050;

            assert_eq!(sequence.get_wrapped_sample_position(22_050), 0);
            assert_eq!(sequence.get_wrapped_sample_position(44_099), 22_049);
            assert_eq!(sequence.get_wrapped_sample_position(44_200), 100);
            assert_eq!(sequence.get_wrapped_sample_position(-100), 21950);

            // test wrong sequence length
            sequence.sequence_length = 0;
            assert_eq!(sequence.get_wrapped_sample_position(44_100), 0);
    }

    #[test]
    fn sequence_sample() {
            let mut sequence = Sequence::default();

            sequence.sequence_length = sequence.beats_to_samples(8.0, 120.0, 44_100.0);

            let note_length = sequence.beats_to_samples(1.0, 120.0, 44_100.0);

            sequence.add_note_event(SeqNoteEvent {
                sample_pos: 0,
                note_data: true,
            });
            sequence.add_note_event(SeqNoteEvent {
                sample_pos: note_length,
                note_data: false,
            });

            assert_eq!(sequence.get_note_events_at_sample(0), vec![SeqNoteEvent{sample_pos: 0, note_data: true}]);
            assert_eq!(sequence.get_note_events_at_sample(note_length), vec![SeqNoteEvent{sample_pos: note_length, note_data: false}]);
            assert_eq!(sequence.get_note_events_at_sample(1000), vec![]);
    }
}
