use nih_plug::prelude::*;
use std::sync::{Arc};

mod sequence;

use crate::sequence::{SeqNoteEvent, Sequence};


#[derive(Enum, Debug, PartialEq)]
enum StepSize {
    #[id = "1"]
    #[name = "1/1"]
    StepSize_1_1,

    #[id = "2"]
    #[name = "1/2"]
    StepSize_1_2,

    #[id = "4"]
    #[name = "1/4"]
    StepSize_1_4,

    #[id = "8"]
    #[name = "1/8"]
    StepSize_1_8,

    #[id = "16"]
    #[name = "1/16"]
    StepSize_1_16,

    #[id = "32"]
    #[name = "1/32"]
    StepSize_1_32,

    #[id = "64"]
    #[name = "1/64"]
    StepSize_1_64,
}

impl StepSize {
    fn get_value(&self) -> f64 {
        match self {
            StepSize::StepSize_1_1 => 1.0/1.0,
            StepSize::StepSize_1_2 => 1.0/2.0,
            StepSize::StepSize_1_4 => 1.0/4.0,
            StepSize::StepSize_1_8 => 1.0/8.0,
            StepSize::StepSize_1_16 => 1.0/16.0,
            StepSize::StepSize_1_32 => 1.0/32.0,
            StepSize::StepSize_1_64 => 1.0/64.0,
        }
    }
}


#[derive(Clone, Default)]
pub struct EuclidianRhythm{
    rhythm: Vec<bool>,
    sequence: Sequence,
}

#[derive(Clone)]
pub struct Euclidian {
    params: Arc<EuclidianParams>,
    rhythms: [EuclidianRhythm;4],
}

#[derive(Params)]
struct VoiceParams {
    #[id = "note_voice_"]
    note: IntParam,

    #[id = "vel_voice_"]
    velocity: IntParam,

    #[id = "num_notes_voice_"]
    num_notes: IntParam,

    #[id = "num_steps_voice_"]
    num_steps: IntParam,

    #[id = "offset_steps_voice_"]
    offset_steps: IntParam,

    #[id = "step_size_voice_"]
    step_size: EnumParam<StepSize>,

    #[id = "enabled_voice_"]
    enabled: BoolParam,
}

#[derive(Params)]
struct EuclidianParams {
    #[nested(array, group = "voices")]
    pub voice_params: [VoiceParams;4],
}

impl Default for EuclidianParams {
    fn default() -> Self {
        Self {
            voice_params: [
                VoiceParams{
                    note: IntParam::new("Voice 1 Note", 36, IntRange::Linear { min: 1, max: 127 }),
                    velocity: IntParam::new("Voice 1 Velocity", 127, IntRange::Linear { min: 1, max: 127 }),
                    num_notes: IntParam::new("Voice 1 Number of Notes", 2, IntRange::Linear { min: 1, max: 64 }),
                    num_steps: IntParam::new("Voice 1 Number of Steps", 8, IntRange::Linear { min: 1, max: 64 }),
                    offset_steps: IntParam::new("Voice 1 Offset of Steps", 0, IntRange::Linear { min: 0, max: 64 }),
                    step_size: EnumParam::new("Voice 1 Step Size", StepSize::StepSize_1_8),
                    enabled: BoolParam::new("Voice 1 Enabled", true),
                },

                VoiceParams{
                    note: IntParam::new("Voice 2 Note", 37, IntRange::Linear { min: 1, max: 127 }),
                    velocity: IntParam::new("Voice 2 Velocity", 127, IntRange::Linear { min: 1, max: 127 }),
                    num_notes: IntParam::new("Voice 2 Number of Notes", 2, IntRange::Linear { min: 1, max: 64 }),
                    num_steps: IntParam::new("Voice 2 Number of Steps", 8, IntRange::Linear { min: 1, max: 64 }),
                    offset_steps: IntParam::new("Voice 2 Offset of Steps", 0, IntRange::Linear { min: 0, max: 64 }),
                    step_size: EnumParam::new("Voice 2 Step Size", StepSize::StepSize_1_8),
                    enabled: BoolParam::new("Voice 2 Enabled", false),
                },

                VoiceParams{
                    note: IntParam::new("Voice 3 Note", 38, IntRange::Linear { min: 1, max: 127 }),
                    velocity: IntParam::new("Voice 3 Velocity", 127, IntRange::Linear { min: 1, max: 127 }),
                    num_notes: IntParam::new("Voice 3 Number of Notes", 2, IntRange::Linear { min: 1, max: 64 }),
                    num_steps: IntParam::new("Voice 3 Number of Steps", 8, IntRange::Linear { min: 1, max: 64 }),
                    offset_steps: IntParam::new("Voice 3 Offset of Steps", 0, IntRange::Linear { min: 0, max: 64 }),
                    step_size: EnumParam::new("Voice 3 Step Size", StepSize::StepSize_1_8),
                    enabled: BoolParam::new("Voice 3 Enabled", false),
                },

                VoiceParams{
                    note: IntParam::new("Voice 4 Note", 39, IntRange::Linear { min: 1, max: 127 }),
                    velocity: IntParam::new("Voice 4 Velocity", 127, IntRange::Linear { min: 1, max: 127 }),
                    num_notes: IntParam::new("Voice 4 Number of Notes", 2, IntRange::Linear { min: 1, max: 64 }),
                    num_steps: IntParam::new("Voice 4 Number of Steps", 8, IntRange::Linear { min: 1, max: 64 }),
                    offset_steps: IntParam::new("Voice 4 Offset of Steps", 0, IntRange::Linear { min: 0, max: 64 }),
                    step_size: EnumParam::new("Voice 4 Step Size", StepSize::StepSize_1_8),
                    enabled: BoolParam::new("Voice 4 Enabled", false),
                },
            ]
        }
    }
}

impl EuclidianParams {}

impl Default for Euclidian {
    fn default() -> Self {
        Self {
            params: Arc::new(EuclidianParams::default()),
            rhythms: Default::default(),
        }
    }
}

fn euclidean_rhythm(num_notes: usize, num_steps: usize, offset_steps: usize) -> Vec<bool> {
    let mut rhythm = vec![false; num_steps];
    let mut bucket = vec![false; num_steps];

    if num_notes > 0 && num_notes <= num_steps {
        let fill_count = num_steps / num_notes;
        let remainder = num_steps % num_notes;

        for i in 0..num_notes {
            rhythm[i * fill_count] = true;
            bucket[i * fill_count] = true;
        }

        let mut offset = offset_steps;
        for _ in 0..remainder {
            if bucket[offset] {
                offset += 1;
            }
            rhythm[offset] = true;
            bucket[offset] = true;
            offset += 1;
        }

        rhythm.rotate_right(offset_steps);
    }

    rhythm
}

impl Euclidian {
    fn update_sequence(sequence: &mut Sequence, rhythm: &Vec<bool>, num_steps: i64, step_size: f64, tempo: f64, sample_rate: f32) {
        let step_len = sequence.beats_to_samples(
            step_size * 4.0,
            tempo,
            sample_rate
        );

        sequence.sequence_length = step_len * num_steps;
        sequence.note_events = vec![];

        for (i, b) in rhythm.iter().enumerate() {
            if *b {
                sequence.add_note_event(SeqNoteEvent {
                    sample_pos: i as i64 * step_len,
                    note_data: true,
                });
                sequence.add_note_event(SeqNoteEvent {
                    sample_pos: (i + 1) as i64 * step_len,
                    note_data: false,
                });
            }
        }
    }

    fn sample_sequence(context: &mut impl ProcessContext<Euclidian>, note: u8, velocity: i32, sample_position: i64, sequence: &Sequence) {
        let wrapped_sample_position = sequence.get_wrapped_sample_position(sample_position);
        for event in sequence.get_note_events_at_sample(wrapped_sample_position).iter() {
            if event.note_data {
                context.send_event(NoteEvent::NoteOn {
                    timing: sample_position as u32,

                    voice_id: None,
                    channel: 0,
                    note: note,
                    velocity: velocity as f32 / 127.0,
                });
            } else {
                context.send_event(NoteEvent::NoteOff {
                    timing: sample_position as u32,

                    voice_id: None,
                    channel: 0,
                    note: note,
                    velocity: velocity as f32 / 127.0,
                });
            }
        }
    }
}


impl Plugin for Euclidian {
    const NAME: &'static str = "Modular::Euclidian";
    const VENDOR: &'static str = "Rochen Sound Collective";
    const URL: &'static str = "https://github.com/rochen-sound-collective/modular_collection";
    const EMAIL: &'static str = "info@example.com";

    const VERSION: &'static str = "0.0.1";

    const DEFAULT_INPUT_CHANNELS: u32 = 0;
    const DEFAULT_OUTPUT_CHANNELS: u32 = 0;

    const DEFAULT_AUX_INPUTS: Option<AuxiliaryIOConfig> = None;
    const DEFAULT_AUX_OUTPUTS: Option<AuxiliaryIOConfig> = None;
    const PORT_NAMES: PortNames = PortNames {
        main_input: None,
        main_output: None,
        aux_inputs: None,
        aux_outputs: None,
    };

    // No MIDI input needed
    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;

    const MIDI_OUTPUT: MidiConfig = MidiConfig::Basic;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    const HARD_REALTIME_ONLY: bool = false;
    type SysExMessage = ();

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(&mut self, bus_config: &BusConfig, buffer_config: &BufferConfig, context: &mut impl InitContext<Self>) -> bool {
        true
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // pretty inefficient to calculate this on every execution

        // Check if the transport is playing and if the time signature information is available
        if context.transport().playing {
            let tempo = context.transport().tempo.unwrap_or(120.0);
            let sample_rate = context.transport().sample_rate;

            // get the step length in quarters
            for (voice_params, mut euclidian) in self.params.voice_params.iter().zip(self.rhythms.iter_mut()){
                if voice_params.enabled.value() {
                    euclidian.rhythm = self::euclidean_rhythm(
                        voice_params.num_notes.value() as usize,
                        voice_params.num_steps.value() as usize,
                        voice_params.offset_steps.value() as usize
                    );
                    Self::update_sequence(&mut euclidian.sequence, &euclidian.rhythm, voice_params.num_steps.value() as i64, voice_params.step_size.value().get_value(), tempo, sample_rate);
                }
            }

            let sample_position_start = context.transport().pos_samples().unwrap_or(0) as i64;

            for i in 0.._buffer.samples() {
                let sample_position = sample_position_start + i as i64;
                for (voice_params, euclidian) in self.params.voice_params.iter().zip(self.rhythms.iter()) {
                    if voice_params.enabled.value() {
                        Self::sample_sequence(context, voice_params.note.value() as u8, voice_params.velocity.value(), sample_position, &euclidian.sequence);
                    }
                }
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Euclidian {
    const CLAP_ID: &'static str = "com.modular.euclidian";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Pattern based arpeggiator.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect, ClapFeature::Utility];
}

impl Vst3Plugin for Euclidian {
    const VST3_CLASS_ID: [u8; 16] = *b"modular.euclid__";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Instrument, Vst3SubCategory::Tools];
}

nih_export_clap!(Euclidian);
nih_export_vst3!(Euclidian);


