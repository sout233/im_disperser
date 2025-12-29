use vizia_plug::vizia::{prelude::*, vg};

pub struct SpectrumView<L>
where
    L: Lens<Target = Vec<f32>>,
{
    data: L,
}

impl<L> SpectrumView<L>
where
    L: Lens<Target = Vec<f32>>,
{
    pub fn new(cx: &mut Context, lens: L) -> Handle<'_, Self> {
        Self { data: lens.clone() }.build(cx, |cx| {
            Binding::new(cx, lens, |cx, _| {
                cx.needs_redraw(cx.current());
            });
        })
    }
}

impl<L> View for SpectrumView<L>
where
    L: Lens<Target = Vec<f32>>,
{
    fn element(&self) -> Option<&'static str> {
        Some("spectrum-view")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let data_vec = self.data.get(cx);
        if data_vec.is_empty() {
            return;
        }

        let background_color = cx.background_color();
        let mut bg_paint = vg::Paint::default();
        bg_paint.set_color(background_color);
        let rect = vg::Rect::from_xywh(bounds.x, bounds.y, bounds.w, bounds.h);
        canvas.draw_rect(&rect, &bg_paint);

        let stroke_color = cx.font_color();

        let max_val = data_vec.iter().copied().fold(0.0f32, f32::max);

        let num_bins = data_vec.len();
        let bin_width = bounds.w / num_bins as f32;

        let mut spectrum_path = vg::Path::new();

        for (i, &val) in data_vec.iter().enumerate() {
            let normalized_val = if max_val > 0.0 { val / max_val } else { 0.0 };
            let bar_height = (normalized_val * bounds.h).clamp(0.0, bounds.h);
            let x = bounds.x + i as f32 * bin_width;
            let y = bounds.y + bounds.h - bar_height;

            spectrum_path.add_rect(vg::Rect::new(x, y, bin_width, bar_height), None);
        }

        let mut fill_paint = vg::Paint::default();
        fill_paint.set_color(stroke_color);
        fill_paint.set_style(vg::PaintStyle::Fill);
        fill_paint.set_anti_alias(true);
        canvas.draw_path(&spectrum_path, &fill_paint);
    }
}
