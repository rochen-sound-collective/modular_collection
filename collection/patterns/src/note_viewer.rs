use std::collections::BTreeSet;
use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::ViziaState;
use crate::active_note::ActiveNoteDefaultData;

#[derive(Default)]
pub struct NoteViewer {

}

impl NoteViewer {
    fn add_note(event: NoteEvent){

    }

    fn remove_note(event: NoteEvent){

    }

}


impl View for NoteViewer {
    fn element(&self) -> Option<&'static str> {
        Some("note-viewer")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        // This spectrum buffer is written to at the end of the process function when the editor is
        // open
        /*let mut spectrum = self.spectrum.lock().unwrap();
        let spectrum = spectrum.read();
        let nyquist = self.sample_rate.load(Ordering::Relaxed) / 2.0;

        // This skips background and border drawing
        let line_width = cx.style.dpi_factor as f32 * 1.5;
        let paint = vg::Paint::color(cx.font_color().cloned().unwrap_or_default().into())
            .with_line_width(line_width);
        for (bin_idx, magnetude) in spectrum.iter().enumerate() {
            // We'll match up the bin's x-coordinate with the filter frequency parameter
            let frequency = (bin_idx as f32 / spectrum.len() as f32) * nyquist;
            let t = self.frequency_range.normalize(frequency);
            if t <= 0.0 || t >= 1.0 {
                continue;
            }

            // Scale this so that 1.0/0 dBFS magnetude is at 80% of the height, the bars begin at
            // -80 dBFS, and that the scaling is linear
            nih_debug_assert!(*magnetude >= 0.0);
            let magnetude_db = nih_plug::util::gain_to_db(*magnetude);
            let height = ((magnetude_db + 80.0) / 100.0).clamp(0.0, 1.0);

            let mut path = vg::Path::new();
            path.move_to(
                bounds.x + (bounds.w * t),
                bounds.y + (bounds.h * (1.0 - height)),
            );
            path.line_to(bounds.x + (bounds.w * t), bounds.y + bounds.h);

            canvas.stroke_path(&mut path, &paint);
        }*/
    }
}