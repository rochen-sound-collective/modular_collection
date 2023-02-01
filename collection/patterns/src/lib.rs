mod active_note;
mod processors;
mod utils;

use crate::processors::ChordPatternProcessor;
use nih_plug::prelude::*;
use std::cmp::max;
use std::sync::{Arc};
use nih_plug::midi::NoteEvent;
use crate::utils::{get_note_of_event, set_note_of_event, get_chord_data, KeyboardMode};

pub struct Patterns {
    params: Arc<PatternsParams>,
    processor: ChordPatternProcessor<Patterns>,
}

#[derive(Params)]
struct PatternsParams {
    #[id = "chord_channel"]
    chord_channel: IntParam,

    #[id = "wrap_threshold"]
    wrap_threshold: IntParam,

    #[id = "auto_threshold"]
    auto_threshold: BoolParam,

    #[id = "octave_range"]
    octave_range: IntParam,

    #[id = "key_mode"]
    key_mode: EnumParam<KeyboardMode>,
}

impl Default for PatternsParams {
    fn default() -> Self {
        Self {
            chord_channel: IntParam::new("Chord Channel", 16, IntRange::Linear { min: 1, max: 16 }),
            wrap_threshold: IntParam::new(
                "Wrap Threshold",
                12,
                IntRange::Linear { min: 1, max: 12 },
            ),
            auto_threshold: BoolParam::new("Auto Threshold", true),
            octave_range: IntParam::new("Octave Range", 12, IntRange::Linear { min: 1, max: 127 }),
            key_mode: EnumParam::new("Keyboard Mode", KeyboardMode::AllKeys),
        }
    }
}

impl PatternsParams{

}

impl Default for Patterns{
    fn default() -> Self {
        Self {
            params: Arc::new(PatternsParams::default()),
            processor: ChordPatternProcessor::default(),
        }
    }
}

impl Patterns {
    fn get_note_event_channel(&self, note_event: &PluginNoteEvent<Patterns>) -> u8 {
        match note_event {
            PluginNoteEvent::<Patterns>::NoteOn { channel, .. }
            | PluginNoteEvent::<Patterns>::NoteOff { channel, .. }
            | PluginNoteEvent::<Patterns>::Choke { channel, .. }
            | PluginNoteEvent::<Patterns>::VoiceTerminated { channel, .. }
            | PluginNoteEvent::<Patterns>::PolyPressure { channel, .. }
            | PluginNoteEvent::<Patterns>::PolyVolume { channel, .. }
            | PluginNoteEvent::<Patterns>::PolyPan { channel, .. }
            | PluginNoteEvent::<Patterns>::PolyTuning { channel, .. }
            | PluginNoteEvent::<Patterns>::PolyVibrato { channel, .. }
            | PluginNoteEvent::<Patterns>::PolyExpression { channel, .. }
            | PluginNoteEvent::<Patterns>::PolyBrightness { channel, .. }
            | PluginNoteEvent::<Patterns>::MidiChannelPressure { channel, .. }
            | PluginNoteEvent::<Patterns>::MidiPitchBend { channel, .. }
            | PluginNoteEvent::<Patterns>::MidiCC { channel, .. }
            | PluginNoteEvent::<Patterns>::MidiProgramChange { channel, .. } => *channel,
            _ => 0,
        }
    }

    fn get_threshold(&self) -> u8 {
        if self.params.auto_threshold.value() {
            max(self.processor.chord.len() as u8, 1) // minimum wrap threshold of 1 to not divide by zero
        }
        else {
            self.params.wrap_threshold.value() as u8
        }
    }
}

impl Plugin for Patterns {
    const NAME: &'static str = "Modular::Patterns";
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

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;

    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;

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
        {
            let mut next_event = context.next_event();
            let mut sample_id = 999;

            let mut other_events: Vec<PluginNoteEvent<Patterns>> = vec![];

            while let Some(event) = next_event {
                if event.timing() != sample_id {
                    let note_events = &mut vec![];
                    self.processor.end_cycle(note_events, sample_id, self.get_threshold(), self.params.octave_range.value() as u8, self.params.key_mode.value());

                    for e in note_events {
                        context.send_event(*e);
                    }
                    // TODO: Modulate other events too
                    for event in other_events.iter() {
                        context.send_event(*event);
                    }
                    other_events.clear();
                    sample_id = event.timing();
                }

                let note_channel = utils::get_channel_of_event::<Patterns>(&event);

                if note_channel == Some((self.params.chord_channel.value() - 1) as u8) {
                    self.processor.process_chord_event(event);
                } else {
                    match event {
                        PluginNoteEvent::<Patterns>::NoteOn { .. } | PluginNoteEvent::<Patterns>::NoteOff { .. } => {
                            self.processor.process_pattern_event(event)
                        }
                        _ => other_events.push(event),
                    }
                }

                next_event = context.next_event();
            }
            // process last chord change. In the above loop the last chord change will not be processed otherwise because the sample_id
            // does not change after the last note.

            let note_events = &mut vec![];
            self.processor.end_cycle(note_events, sample_id, self.get_threshold(), self.params.octave_range.value() as u8, self.params.key_mode.value());

            for e in note_events {
                context.send_event(*e);
            }
            // TODO: Modulate other events too
            for note_event in other_events.iter() {
                if let Some(raw_note) = get_note_of_event::<Patterns>(&note_event) {
                    let chord_data = get_chord_data(&self.processor.chord.iter().cloned().collect(), raw_note, self.get_threshold(), self.params.octave_range.value() as u8);
                    if let Some(triggered_note) = chord_data.triggered_note {
                        context.send_event(set_note_of_event::<Patterns>(note_event, triggered_note));
                    }
                }
            }

            other_events.clear();

        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Patterns {
    const CLAP_ID: &'static str = "com.modular.patterns";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Pattern based arpeggiator.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect, ClapFeature::Utility];
}

impl Vst3Plugin for Patterns {
    const VST3_CLASS_ID: [u8; 16] = *b"modular.patterns";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Instrument, Vst3SubCategory::Tools];
}

nih_export_clap!(Patterns);
nih_export_vst3!(Patterns);

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