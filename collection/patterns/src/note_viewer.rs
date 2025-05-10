use std::sync::{Arc, Mutex};
use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use crate::active_note::ActiveNoteDefaultData;

#[derive(Lens)]
pub struct NoteViewer {
    active_notes: Arc<Mutex<Vec<ActiveNoteDefaultData>>>,
    chord_channel: u8,
    pixels_per_beat: f32,
}

impl NoteViewer {
    pub fn new(
        cx: &mut Context,
        active_notes: Arc<Mutex<Vec<ActiveNoteDefaultData>>>,
        chord_channel: u8,
    ) -> Handle<Self> {
        Self {
            active_notes,
            chord_channel,
            pixels_per_beat: 100.0, // Default zoom level
        }.build(cx, |cx| {
            // Add zoom controls
            HStack::new(cx, |cx| {
                Button::new(cx, |cx| cx.emit(NoteViewerEvent::ZoomIn), |cx| Label::new(cx, "+"));
                Button::new(cx, |cx| cx.emit(NoteViewerEvent::ZoomOut), |cx| Label::new(cx, "-"));
            });
        })
    }

    fn note_to_y(&self, note: u8) -> f32 {
        // Map MIDI note (0-127) to vertical position (C0 at bottom)
        let note_height = 12.0; // Height per octave
        400.0 - (note as f32 * note_height / 12.0)
    }

    fn is_black_key(note: u8) -> bool {
        matches!(note % 12, 1 | 3 | 6 | 8 | 10)
    }
}

pub enum NoteViewerEvent {
    ZoomIn,
    ZoomOut,
}

impl View for NoteViewer {
    fn element(&self) -> Option<&'static str> {
        Some("note-viewer")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            NoteViewerEvent::ZoomIn => self.pixels_per_beat *= 1.2,
            NoteViewerEvent::ZoomOut => self.pixels_per_beat /= 1.2,
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        let current_time = cx.transport().pos_beats() as f32;
        let notes = self.active_notes.lock().unwrap();

        // Draw piano keys background
        for note in 0..127 {
            let y = self.note_to_y(note);
            let color = if Self::is_black_key(note) {
                Color::rgb(50, 50, 50)
            } else {
                Color::rgb(200, 200, 200)
            };
            canvas.fill_rect(0.0, y, bounds.w, 12.0, color);
        }

        // Draw active notes
        for note_data in notes.iter() {
            let start_x = note_data.start_time_beats as f32 * self.pixels_per_beat;
            let end_x = note_data.end_time_beats.unwrap_or(current_time) * self.pixels_per_beat;
            let width = end_x - start_x;

            let color = if note_data.channel == self.chord_channel {
                Color::rgb(100, 200, 100) // Green for chords
            } else {
                Color::rgb(200, 100, 100) // Red for patterns
            };

            let y = self.note_to_y(note_data.note);
            canvas.fill_rect(start_x, y, width, 12.0, color);
        }

        // Draw playhead
        let playhead_x = current_time * self.pixels_per_beat;
        canvas.fill_rect(playhead_x, 0.0, 2.0, bounds.h, Color::rgb(255, 255, 0));
    }
}