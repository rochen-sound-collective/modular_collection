use nih_plug::prelude::*;
use std::sync::{Arc};


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

pub struct Euclidian {
    params: Arc<EuclidianParams>,
    rhythms: Vec<bool>,
}

#[derive(Params)]
struct EuclidianParams {
    #[id = "voice1_note"]
    voice1_note: IntParam,

    #[id = "voice1_vel"]
    voice1_vel: IntParam,

    #[id = "voice1_num_notes"]
    voice1_num_notes: IntParam,

    #[id = "voice1_num_steps"]
    voice1_num_steps: IntParam,

    #[id = "voice1_offset_steps"]
    voice1_offset_steps: IntParam,
}

impl Default for EuclidianParams {
    fn default() -> Self {
        Self {
            voice1_note: IntParam::new("Voice 1 Note", 16, IntRange::Linear { min: 1, max: 127 }),
            voice1_vel: IntParam::new("Voice 1 Velocity", 127, IntRange::Linear { min: 1, max: 127 }),
            voice1_num_notes: IntParam::new("Voice 1 Number of Notes", 2, IntRange::Linear { min: 1, max: 64 }),
            voice1_num_steps: IntParam::new("Voice 1 Number of Steps", 8, IntRange::Linear { min: 1, max: 64 }),
            voice1_offset_steps: IntParam::new("Voice 1 Offset of Steps", 0, IntRange::Linear { min: 1, max: 64 }),
        }
    }
}

impl EuclidianParams {}

impl Default for Euclidian {
    fn default() -> Self {
        Self {
            params: Arc::new(EuclidianParams::default()),
            rhythms: vec![],
        }
    }
}

impl Euclidian {
    fn calculate_rhythms(&mut self) {
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

        self.rhythms = euclidean_rhythm(self.params.voice1_num_notes.value() as usize,
                                        self.params.voice1_num_steps.value() as usize,
                                        self.params.voice1_offset_steps.value() as usize)
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
    const MIDI_INPUT: MidiConfig = MidiConfig::None;

    const MIDI_OUTPUT: MidiConfig = MidiConfig::Basic;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    const HARD_REALTIME_ONLY: bool = false;
    type SysExMessage = ();

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // pretty inefficient to calculate this on every execution
        //self.calculate_rhythms();

        // Check if the transport is playing and if the time signature information is available
        if context.transport().playing
            //&& context.transport().time_sig_numerator.is_some()
            //&& context.transport().time_sig_denominator.is_some()
        {
            // Calculate the PPQ (Pulses Per Quarter) value
            //let ppq = context.transport().time_sig_numerator.unwrap() * context.transport().time_sig_denominator.unwrap();

            //if context.transport().pos_beats().map(|beats| beats.fract() == 0.0).unwrap_or(false) {

            // Check if the current position is at the start of a bar
            if let Some(beats) = context.transport().pos_beats() {
                if beats.fract() == 0.0 {
                    // Send the MIDI note event
                    let sample_position = context.transport().pos_samples().unwrap_or(0) as u32;
                    context.send_event(NoteEvent::NoteOn {
                        timing: sample_position,
                        voice_id: None,
                        channel: 0,
                        note: self.params.voice1_note.value() as u8,
                        velocity: self.params.voice1_vel.value() as f32 / 127.0,
                    });

                    // Calculate the sample position for the note-off event (1/8th note after the note-on event)
                    let sample_position_note_off = sample_position + (context.transport().sample_rate * 60.0 / context.transport().tempo.unwrap_or(120.0) as f32 * 0.125) as u32;

                    // Send the MIDI note-off event
                    context.send_event(NoteEvent::NoteOff {
                        timing: sample_position_note_off,
                        voice_id: None,
                        channel: 0,
                        note: self.params.voice1_note.value() as u8,
                        velocity: self.params.voice1_vel.value() as f32 / 127.0,
                    });
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

/*
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
*/