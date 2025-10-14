use crate::active_note::ActiveNoteDefaultData;
use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::vg::{self, Paint, Path};
use std::sync::{Arc, Mutex};

pub struct NoteViewer {
    chord_channel: u8,
    pixels_per_beat: f32,
    active_notes: Arc<Mutex<Vec<ActiveNoteDefaultData>>>,
}

impl NoteViewer {
    pub fn new(
        cx: &mut Context,
        chord_channel: u8,
        active_notes: Arc<Mutex<Vec<ActiveNoteDefaultData>>>,
    ) -> Handle<Self> {
        Self {
            chord_channel,
            pixels_per_beat: 100.0,
            active_notes,
        }
        .build(cx, |_| {})
        .width(Pixels(800.0))
        .height(Stretch(1.0))
    }

    fn note_to_y(&self, note: u8) -> f32 {
        (127 - note) as f32 * 12.0
    }

    fn is_black_key(note: u8) -> bool {
        matches!(note % 12, 1 | 3 | 6 | 8 | 10)
    }
}

impl View for NoteViewer {
    fn element(&self) -> Option<&'static str> {
        Some("note-viewer")
    }

    fn event(&mut self, _cx: &mut EventContext, _event: &mut Event) {}

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        let notes = self.active_notes.lock().unwrap();
        //nih_dbg!(&notes);

        // Draw piano keys, highlighting currently held notes
        for note in 0..=127 {
            let y = self.note_to_y(note as u8);
            let mut path = Path::new();
            path.rect(0.0, y, bounds.w, 12.0);

            let is_pressed = notes.iter().any(|d| d.note == note as u8);
            let fill_color = if is_pressed {
                vg::Color::rgb(100, 200, 100)
            } else if Self::is_black_key(note as u8) {
                vg::Color::rgb(50, 50, 50)
            } else {
                vg::Color::rgb(200, 200, 200)
            };
            let fill_paint = Paint::color(fill_color);
            let stroke_paint = Paint::color(vg::Color::rgb(30, 30, 30)).with_line_width(1.0);

            canvas.fill_path(&path, &fill_paint);
            canvas.stroke_path(&path, &stroke_paint);
        }

        // Draw a fixed playhead at left edge
        let mut head = Path::new();
        head.rect(0.0, 0.0, 2.0, bounds.h);
        canvas.fill_path(&head, &Paint::color(vg::Color::rgb(255, 255, 0)));
    }
}
