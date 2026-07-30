use nih_plug::prelude::Param;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};
use vizia_plug::vizia::prelude::*;
use vizia_plug::vizia::vg;
use vizia_plug::vizia::vg::Point;

use vizia_plug::widgets::param_base::ParamWidgetBase;
use vizia_plug::widgets::util::ModifiersExt;
use vizia_plug::widgets::RawParamEvent;

// Normal drag changes 0.2% of the normalized range per physical pixel. For the 0..100 Amount
// parameter this is 0.2 Amount units per pixel before integer rounding, or roughly 500 pixels
// for the complete range.
const DEFAULT_DRAG_SCALAR: f32 = 0.002;
// Shift drag is 25x finer than normal drag: 0.008% of the normalized range per pixel. That is
// 0.00008 normalized units per pixel and roughly 12,500 pixels for the complete Amount range.
const FINE_DRAG_MULTIPLIER: f32 = 0.04;
const DEFAULT_WHEEL_SCALAR: f32 = 0.005;
const HOST_UPDATE_INTERVAL: Duration = Duration::from_micros(16_667);

#[derive(Debug, Clone, Default)]
pub struct DragStatus {
    previous_y: Option<f32>,
    normalized_value: f32,
    last_host_commit_at: Option<Instant>,
    last_committed_value: Option<f32>,
}

#[derive(Lens)]
pub struct ParamKnob {
    param_base: ParamWidgetBase,
    text_input_active: bool,
    drag_status: Option<DragStatus>,
    drag_scalar: f32,
    wheel_scalar: f32,
    preview_normalized_value: f32,
    #[lens(ignore)]
    published_display_value: String,
    #[lens(ignore)]
    display_label_visible: bool,
    #[lens(ignore)]
    visual_entity: Rc<Cell<Entity>>,
    #[lens(ignore)]
    value_label_entity: Rc<Cell<Entity>>,
    #[lens(ignore)]
    display_state: Rc<RefCell<ParamValueDisplayState>>,
}

enum ParamKnobEvent {
    CancelTextInput,
    TextInput(String),
}

enum KnobVisualEvent {
    SetValue(f32),
}

#[derive(Default)]
struct ParamValueDisplayState {
    text: String,
    visible: bool,
}

impl ParamKnob {
    pub fn new<L, Params, P, FMap>(
        cx: &mut Context,
        params: L,
        params_to_param: FMap,
        centered: bool,
    ) -> Handle<'_, Self>
    where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        let param_base = ParamWidgetBase::new(cx, params, params_to_param);
        let initial_normalized_value = param_base.unmodulated_normalized_value();
        let initial_display_value =
            format_preview_display_value(&param_base, initial_normalized_value);
        let visual_entity = Rc::new(Cell::new(Entity::null()));
        let value_label_entity = Rc::new(Cell::new(Entity::null()));
        let display_state = Rc::new(RefCell::new(ParamValueDisplayState {
            text: initial_display_value.clone(),
            visible: false,
        }));
        let visual_entity_for_build = visual_entity.clone();
        let value_label_entity_for_build = value_label_entity.clone();
        let display_state_for_build = display_state.clone();

        Self {
            param_base,
            text_input_active: false,
            drag_status: None,
            drag_scalar: DEFAULT_DRAG_SCALAR,
            wheel_scalar: DEFAULT_WHEEL_SCALAR,
            preview_normalized_value: initial_normalized_value,
            published_display_value: initial_display_value,
            display_label_visible: false,
            visual_entity,
            value_label_entity: value_label_entity.clone(),
            display_state,
        }
        .build(
            cx,
            ParamWidgetBase::build_view(params, params_to_param, move |cx, param_data| {
                let display_value_lens = param_data.make_lens(|param| {
                    param.normalized_value_to_string(param.unmodulated_normalized_value(), true)
                });

                Binding::new(cx, ParamKnob::text_input_active, move |cx, active| {
                    if active.get(cx) {
                        Textbox::new(cx, display_value_lens)
                            .class("value-entry")
                            .on_submit(|cx, string, success| {
                                if success {
                                    cx.emit(ParamKnobEvent::TextInput(string));
                                } else {
                                    cx.emit(ParamKnobEvent::CancelTextInput);
                                }
                            })
                            .on_cancel(|cx| cx.emit(ParamKnobEvent::CancelTextInput))
                            .on_build(|cx| {
                                cx.emit(TextEvent::StartEdit);
                                cx.emit(TextEvent::SelectAll);
                            })
                            .width(Stretch(1.0))
                            .height(Stretch(1.0));
                    }
                });

                let visual = KnobVisual::new(cx, centered, initial_normalized_value)
                    .width(Stretch(1.0))
                    .height(Stretch(1.0))
                    .hoverable(false);
                visual_entity_for_build.set(visual.entity());

                // Keep one readout entity per knob for the whole lifetime of the view. Rebuilding
                // it from a drag-state binding made its layout and hit-test state race with mouse
                // events, which could move a readout over a different knob.
                let label = ParamValueDisplay::new(cx, display_state_for_build)
                    .class("param-value-label")
                    .position_type(PositionType::Absolute)
                    .alignment(Alignment::TopCenter)
                    .hoverable(false)
                    .width(Pixels(100.0))
                    .height(Pixels(50.0));
                value_label_entity_for_build.set(label.entity());
            }),
        )
    }
}

impl ParamKnob {
    fn update_preview(&mut self, cx: &mut EventContext, normalized_value: f32) {
        self.preview_normalized_value = normalized_value.clamp(0.0, 1.0);
        let visual_entity = self.visual_entity.get();
        if !visual_entity.is_null() {
            cx.emit_to(
                visual_entity,
                KnobVisualEvent::SetValue(self.preview_normalized_value),
            );
        }
    }

    fn refresh_display_label(&mut self, cx: &mut EventContext, visible: bool) {
        let next_text =
            format_preview_display_value(&self.param_base, self.preview_normalized_value);
        if !display_update_needed(
            &self.published_display_value,
            &next_text,
            self.display_label_visible,
            visible,
        ) {
            return;
        }

        self.published_display_value = next_text.clone();
        self.display_label_visible = visible;

        {
            let mut display_state = self.display_state.borrow_mut();
            display_state.text = next_text;
            display_state.visible = visible;
        }

        let label_entity = self.value_label_entity.get();
        if !label_entity.is_null() {
            cx.with_current(label_entity, |cx| cx.needs_redraw());
        }
    }

    fn update_preview_from_parameter(&mut self, cx: &mut EventContext) {
        let host_value = self.param_base.unmodulated_normalized_value();
        let normalized_value = preview_value_after_host_update(
            self.drag_status.is_some(),
            self.preview_normalized_value,
            host_value,
        );
        if self.drag_status.is_none() {
            self.update_preview(cx, normalized_value);
        }
    }

    fn snapped_normalized_value(&self, normalized_value: f32) -> f32 {
        self.param_base
            .preview_normalized(self.param_base.preview_plain(normalized_value))
    }
}

impl View for ParamKnob {
    fn element(&self) -> Option<&'static str> {
        Some("param-knob")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e, meta| match e {
            ParamKnobEvent::TextInput(s) => {
                if let Some(val) = self.param_base.string_to_normalized_value(s) {
                    self.param_base.begin_set_parameter(cx);
                    self.param_base.set_normalized_value(cx, val);
                    self.param_base.end_set_parameter(cx);
                    self.update_preview(cx, self.snapped_normalized_value(val));
                    cx.needs_redraw();
                }
                self.text_input_active = false;
                cx.set_active(false);
                meta.consume();
            }
            ParamKnobEvent::CancelTextInput => {
                self.text_input_active = false;
                cx.set_active(false);
                meta.consume();
            }
        });

        event.map(|raw_event: &RawParamEvent, _| match raw_event {
            RawParamEvent::ParametersChanged if self.drag_status.is_none() => {
                self.update_preview_from_parameter(cx);
                cx.needs_redraw();
            }
            _ => {}
        });

        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left)
            | WindowEvent::MouseTripleClick(MouseButton::Left) => {
                if cx.modifiers().alt() {
                    self.param_base.begin_set_parameter(cx);
                    self.param_base
                        .set_normalized_value(cx, self.param_base.default_normalized_value());
                    self.param_base.end_set_parameter(cx);
                    self.update_preview(cx, self.param_base.default_normalized_value());
                    cx.needs_redraw();
                    meta.consume();
                } else if cx.modifiers().command() {
                    self.text_input_active = true;
                    cx.set_active(true);
                    meta.consume();
                } else if !self.text_input_active {
                    cx.capture();
                    cx.focus();
                    cx.set_active(true);
                    self.drag_status = Some(DragStatus {
                        // A valid MouseDown coordinate is the drag baseline, so even a short
                        // gesture containing only one coalesced MouseMove changes the value.
                        // Fall back to establishing the baseline on MouseMove only if the
                        // backend supplied an invalid coordinate.
                        previous_y: initial_drag_baseline(cx.mouse().cursor_y),
                        normalized_value: self.param_base.unmodulated_normalized_value(),
                        last_host_commit_at: None,
                        last_committed_value: None,
                    });
                    // Establish both visual channels from the same host value before the first
                    // drag delta. This avoids starting a gesture from a stale arc/readout value.
                    self.update_preview(cx, self.param_base.unmodulated_normalized_value());
                    cx.needs_redraw();
                    self.refresh_display_label(cx, true);

                    self.param_base.begin_set_parameter(cx);
                    meta.consume();
                }
            }

            WindowEvent::MouseDoubleClick(MouseButton::Left) => {
                self.param_base.begin_set_parameter(cx);
                self.param_base
                    .set_normalized_value(cx, self.param_base.default_normalized_value());
                self.param_base.end_set_parameter(cx);
                self.update_preview(cx, self.param_base.default_normalized_value());
                cx.needs_redraw();
                meta.consume();
            }

            WindowEvent::MouseDown(MouseButton::Right) => {
                self.text_input_active = true;
                cx.set_active(true);
                meta.consume();
            }

            WindowEvent::MouseUp(MouseButton::Left) => {
                if self.drag_status.is_some() {
                    let final_value = commit_value_on_mouse_up(
                        self.drag_status
                            .as_ref()
                            .and_then(|status| status.last_committed_value),
                        self.preview_normalized_value,
                    );
                    self.param_base.set_normalized_value(cx, final_value);
                    self.update_preview(cx, self.snapped_normalized_value(final_value));
                    cx.needs_redraw();
                    self.drag_status.take();
                    self.refresh_display_label(cx, false);
                    cx.release();
                    cx.set_active(false);
                    self.param_base.end_set_parameter(cx);
                    meta.consume();
                }
            }

            WindowEvent::MouseMove(_, y) => {
                if self.drag_status.is_some() {
                    let now = Instant::now();
                    let (normalized_value, should_commit) = {
                        let status = self.drag_status.as_mut().unwrap();
                        let Some(delta_y) = next_drag_delta(&mut status.previous_y, *y) else {
                            meta.consume();
                            return;
                        };

                        status.normalized_value = relative_drag_value(
                            status.normalized_value,
                            delta_y,
                            self.drag_scalar,
                            cx.modifiers().shift(),
                        );
                        let should_commit = host_commit_due(
                            status
                                .last_host_commit_at
                                .map(|last| now.saturating_duration_since(last)),
                        );
                        (status.normalized_value, should_commit)
                    };

                    // Keep the visual value continuous even when the host parameter is stepped
                    // or the host only accepts updates at a lower rate.
                    self.update_preview(cx, normalized_value);
                    self.refresh_display_label(cx, true);
                    cx.needs_redraw();

                    if should_commit {
                        self.param_base.set_normalized_value(cx, normalized_value);
                        let status = self.drag_status.as_mut().unwrap();
                        status.last_host_commit_at = Some(now);
                        status.last_committed_value = Some(normalized_value);
                    }

                    meta.consume();
                }
            }

            WindowEvent::MouseScroll(_, y) => {
                if *y != 0.0 && self.drag_status.is_none() {
                    self.param_base.begin_set_parameter(cx);

                    let use_finer_steps = cx.modifiers().shift();
                    let mut current_value = self.param_base.unmodulated_normalized_value();

                    if *y > 0.0 {
                        current_value = self
                            .param_base
                            .next_normalized_step(current_value, use_finer_steps);
                    } else {
                        current_value = self
                            .param_base
                            .previous_normalized_step(current_value, use_finer_steps);
                    }

                    self.param_base.set_normalized_value(cx, current_value);
                    self.param_base.end_set_parameter(cx);
                    self.update_preview_from_parameter(cx);
                    cx.needs_redraw();
                    meta.consume();
                }
            }
            _ => {}
        });
    }
}

/// The visual part of a parameter knob. This view has no parameter pointer and no mouse logic;
/// it only redraws itself when the input controller sends a continuous preview value.
struct KnobVisual {
    normalized_value: f32,
    centered: bool,
}

impl KnobVisual {
    fn new(cx: &mut Context, centered: bool, normalized_value: f32) -> Handle<'_, Self> {
        Self {
            normalized_value: normalized_value.clamp(0.0, 1.0),
            centered,
        }
        .build(cx, |_| {})
    }
}

impl View for KnobVisual {
    fn element(&self) -> Option<&'static str> {
        Some("knob-visual")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|visual_event: &KnobVisualEvent, _| match visual_event {
            KnobVisualEvent::SetValue(value) => {
                self.normalized_value = value.clamp(0.0, 1.0);
                cx.needs_redraw();
            }
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &vg::Canvas) {
        let foreground_color = Color::rgba(0, 255, 60, 50);
        let background_color = Color::transparent();
        let tick_color = Color::black();
        let bounds = cx.bounds();

        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let center_x = bounds.x + bounds.w / 2.0;
        let center_y = bounds.y + bounds.h / 2.0;
        let radius = bounds.w.min(bounds.h) / 2.0;
        let stroke_width = radius * 0.1;
        let draw_radius = radius - stroke_width / 2.0;

        let angle_offset = -90.0;
        let start_angle_deg = -150.0 + angle_offset;
        let end_angle_deg = 150.0 + angle_offset;
        let geometry = arc_geometry(self.normalized_value, start_angle_deg, end_angle_deg);

        let oval = vg::Rect::new(
            center_x - draw_radius,
            center_y - draw_radius,
            center_x + draw_radius,
            center_y + draw_radius,
        );

        let mut paint = vg::Paint::default();
        paint.set_color(Color::rgb(241, 255, 243));
        paint.set_style(vg::PaintStyle::Fill);
        canvas.draw_circle(Point::new(center_x, center_y), draw_radius, &paint);

        let mut background_paint = vg::Paint::default();
        background_paint.set_color(background_color);
        background_paint.set_stroke_width(stroke_width);
        background_paint.set_stroke_cap(vg::PaintCap::Butt);
        background_paint.set_style(vg::PaintStyle::Stroke);
        background_paint.set_anti_alias(true);
        canvas.draw_arc(
            &oval,
            start_angle_deg,
            end_angle_deg - start_angle_deg,
            false,
            &background_paint,
        );

        let mut foreground_paint = vg::Paint::default();
        foreground_paint.set_color(foreground_color);
        foreground_paint.set_stroke_width(stroke_width);
        foreground_paint.set_stroke_cap(vg::PaintCap::Butt);
        foreground_paint.set_style(vg::PaintStyle::Stroke);
        foreground_paint.set_anti_alias(true);
        let _ = self.centered;
        canvas.draw_arc(
            &oval,
            start_angle_deg,
            geometry.active_sweep_deg,
            false,
            &foreground_paint,
        );

        let current_angle_rad = geometry.tick_angle_deg.to_radians();
        let tick_outer_radius = radius;
        let tick_inner_radius = radius - stroke_width;
        let tick_x0 = center_x + current_angle_rad.cos() * tick_inner_radius;
        let tick_y0 = center_y + current_angle_rad.sin() * tick_inner_radius;
        let tick_x1 = center_x + current_angle_rad.cos() * tick_outer_radius;
        let tick_y1 = center_y + current_angle_rad.sin() * tick_outer_radius;

        let mut paint_tick = vg::Paint::default();
        paint_tick.set_color(tick_color);
        paint_tick.set_stroke_width(4.0);
        paint_tick.set_stroke_cap(vg::PaintCap::Butt);
        paint_tick.set_style(vg::PaintStyle::Stroke);

        let mut path = vg::Path::new();
        path.move_to((tick_x0, tick_y0));
        path.line_to((tick_x1, tick_y1));
        canvas.draw_path(&path, &paint_tick);
    }
}

/// A fixed-size value readout which only redraws its own canvas. Unlike a `Label`, updating this
/// view does not call `Context::set_text()`, so live dragging never causes a text reflow or a
/// layout pass for the surrounding editor.
struct ParamValueDisplay {
    display_state: Rc<RefCell<ParamValueDisplayState>>,
    font: vg::Font,
}

impl ParamValueDisplay {
    fn new(
        cx: &mut Context,
        display_state: Rc<RefCell<ParamValueDisplayState>>,
    ) -> Handle<'_, Self> {
        Self {
            display_state,
            font: readout_font(),
        }
        .build(cx, |_| {})
    }
}

impl View for ParamValueDisplay {
    fn element(&self) -> Option<&'static str> {
        Some("param-value-display")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &vg::Canvas) {
        let display_state = self.display_state.borrow();
        if !display_state.visible {
            return;
        }

        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        cx.draw_background(canvas);
        cx.draw_shadows(canvas);
        cx.draw_border(canvas);

        let mut paint = vg::Paint::default();
        paint.set_color(Color::rgb(18, 23, 19));
        paint.set_anti_alias(true);

        canvas.draw_str_align(
            &display_state.text,
            Point::new(bounds.x + bounds.w / 2.0, bounds.y + bounds.h / 2.0 + 4.0),
            &self.font,
            &paint,
            vg::utils::text_utils::Align::Center,
        );
    }
}

fn readout_font() -> vg::Font {
    let font_manager = vg::FontMgr::default();
    let normal = vg::FontStyle::normal();
    let typeface = font_manager
        .match_family_style("JetBrains Mono", normal)
        .or_else(|| font_manager.match_family_style("Menlo", normal))
        .or_else(|| font_manager.match_family_style("monospace", normal))
        .or_else(|| font_manager.legacy_make_typeface(None::<&str>, normal))
        .expect("the macOS system font manager must provide a fallback typeface");

    vg::Font::from_typeface(typeface, Some(24.0))
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ArcGeometry {
    active_sweep_deg: f32,
    tick_angle_deg: f32,
}

fn arc_geometry(normalized_value: f32, start_angle_deg: f32, end_angle_deg: f32) -> ArcGeometry {
    let value = normalized_value.clamp(0.0, 1.0);
    let sweep_angle_deg = end_angle_deg - start_angle_deg;

    ArcGeometry {
        active_sweep_deg: value * sweep_angle_deg,
        tick_angle_deg: start_angle_deg + value * sweep_angle_deg,
    }
}

fn relative_drag_value(current_value: f32, delta_y: f32, drag_scalar: f32, fine: bool) -> f32 {
    // Keep the fine adjustment consistent with Vizia's built-in Knob. Applying each mouse
    // movement to the previous continuous value also preserves fractional movement for stepped
    // parameters instead of repeatedly starting from the snapped parameter value.
    let fine_multiplier = if fine { FINE_DRAG_MULTIPLIER } else { 1.0 };
    (current_value - delta_y * drag_scalar * fine_multiplier).clamp(0.0, 1.0)
}

fn format_preview_display_value(param_base: &ParamWidgetBase, normalized_value: f32) -> String {
    if let Some(step_count) = param_base.step_count() {
        // Amount is intentionally still an IntParam. Its preview is continuous only inside the
        // editor, so use the integer range's step count to format a temporary one-decimal value.
        format!(
            "{:.1}",
            normalized_value.clamp(0.0, 1.0) * step_count as f32
        )
    } else if param_base.name() == "Frequency" {
        // The regular Frequency formatter includes a note name rounded to the nearest semitone.
        // That is useful for text entry, but it makes the live drag readout look stepped. During
        // dragging, show only the continuously interpolated numeric frequency instead.
        format!("{:.2} Hz", param_base.preview_plain(normalized_value))
    } else if param_base.name() == "Spread" {
        format!(
            "{:.2}{}",
            param_base.preview_plain(normalized_value),
            param_base.unit()
        )
    } else {
        param_base.normalized_value_to_string(normalized_value, true)
    }
}

fn next_drag_delta(previous_y: &mut Option<f32>, current_y: f32) -> Option<f32> {
    if !current_y.is_finite() || current_y < 0.0 {
        return None;
    }

    previous_y
        .replace(current_y)
        .map(|previous| current_y - previous)
}

fn initial_drag_baseline(cursor_y: f32) -> Option<f32> {
    if cursor_y.is_finite() && cursor_y >= 0.0 {
        Some(cursor_y)
    } else {
        None
    }
}

fn host_commit_due(elapsed_since_last_commit: Option<Duration>) -> bool {
    elapsed_since_last_commit.map_or(true, |elapsed| elapsed >= HOST_UPDATE_INTERVAL)
}

fn commit_value_on_mouse_up(_last_committed_value: Option<f32>, preview_value: f32) -> f32 {
    preview_value
}

fn preview_value_after_host_update(dragging: bool, preview_value: f32, host_value: f32) -> f32 {
    if dragging {
        preview_value
    } else {
        host_value
    }
}

fn display_update_needed(
    previous_text: &str,
    next_text: &str,
    previous_visible: bool,
    next_visible: bool,
) -> bool {
    previous_text != next_text || previous_visible != next_visible
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    #[test]
    fn drag_value_uses_the_mouse_delta() {
        let value = super::relative_drag_value(0.5, -10.0, super::DEFAULT_DRAG_SCALAR, false);

        assert!((value - 0.52).abs() < f32::EPSILON);
    }

    #[test]
    fn shift_drag_uses_a_finer_step() {
        let value = super::relative_drag_value(0.5, -10.0, super::DEFAULT_DRAG_SCALAR, true);

        assert!((value - 0.5008).abs() < f32::EPSILON);
    }

    #[test]
    fn drag_value_is_clamped_to_the_normalized_range() {
        assert_eq!(
            super::relative_drag_value(0.99, -1000.0, super::DEFAULT_DRAG_SCALAR, false),
            1.0
        );
        assert_eq!(
            super::relative_drag_value(0.01, 1000.0, super::DEFAULT_DRAG_SCALAR, false),
            0.0
        );
    }

    #[test]
    fn relative_drag_preserves_fractional_motion() {
        let first_value = super::relative_drag_value(0.5, -1.0, super::DEFAULT_DRAG_SCALAR, false);
        let second_value =
            super::relative_drag_value(first_value, -1.0, super::DEFAULT_DRAG_SCALAR, false);

        assert!((second_value - 0.504).abs() < f32::EPSILON);
    }

    #[test]
    fn first_mouse_move_uses_the_mouse_down_baseline() {
        let mut previous_y = super::initial_drag_baseline(300.0);

        assert_eq!(
            super::next_drag_delta(&mut previous_y, 290.0),
            Some(-10.0),
            "a short first drag must not lose its only MouseMove event"
        );
    }

    #[test]
    fn invalid_mouse_down_position_defers_baseline_until_mouse_move() {
        let mut previous_y = super::initial_drag_baseline(f32::NAN);

        assert_eq!(super::next_drag_delta(&mut previous_y, 300.0), None);
        assert_eq!(previous_y, Some(300.0));
    }

    #[test]
    fn invalid_mouse_positions_are_ignored_without_changing_the_baseline() {
        let mut previous_y = None;

        assert_eq!(super::next_drag_delta(&mut previous_y, -1.0), None);
        assert_eq!(super::next_drag_delta(&mut previous_y, f32::NAN), None);
        assert_eq!(super::next_drag_delta(&mut previous_y, f32::INFINITY), None);
        assert_eq!(previous_y, None);

        assert_eq!(super::next_drag_delta(&mut previous_y, 300.0), None);
        assert_eq!(super::next_drag_delta(&mut previous_y, 290.0), Some(-10.0));
    }

    #[test]
    fn host_updates_are_coalesced_to_about_60_hz() {
        assert!(super::host_commit_due(None));
        assert!(!super::host_commit_due(Some(Duration::from_millis(16))));
        assert!(super::host_commit_due(Some(Duration::from_micros(16_667))));
    }

    #[test]
    fn final_value_is_always_flushed() {
        assert_eq!(super::commit_value_on_mouse_up(Some(0.37), 0.42), 0.42);
        assert_eq!(super::commit_value_on_mouse_up(None, 0.42), 0.42);
    }

    #[test]
    fn drag_preview_wins_over_host_echo() {
        assert_eq!(
            super::preview_value_after_host_update(true, 0.42, 0.4),
            0.42
        );
        assert_eq!(
            super::preview_value_after_host_update(false, 0.42, 0.4),
            0.4
        );
    }

    #[test]
    fn display_update_is_requested_when_live_value_changes() {
        assert!(super::display_update_needed(
            "1005.63 Hz",
            "1006.12 Hz",
            true,
            true,
        ));
        assert!(super::display_update_needed(
            "1005.63 Hz",
            "1005.63 Hz",
            false,
            true,
        ));
        assert!(!super::display_update_needed(
            "1005.63 Hz",
            "1005.63 Hz",
            true,
            true,
        ));
    }

    #[test]
    fn arc_geometry_tracks_the_full_normalized_range() {
        assert_eq!(
            super::arc_geometry(0.0, -240.0, 60.0),
            super::ArcGeometry {
                active_sweep_deg: 0.0,
                tick_angle_deg: -240.0,
            }
        );
        assert_eq!(
            super::arc_geometry(0.5, -240.0, 60.0),
            super::ArcGeometry {
                active_sweep_deg: 150.0,
                tick_angle_deg: -90.0,
            }
        );
        assert_eq!(
            super::arc_geometry(1.0, -240.0, 60.0),
            super::ArcGeometry {
                active_sweep_deg: 300.0,
                tick_angle_deg: 60.0,
            }
        );
    }

    #[test]
    fn arc_geometry_uses_the_continuous_drag_preview() {
        let preview = super::relative_drag_value(0.5, -1.0, super::DEFAULT_DRAG_SCALAR, false);
        let geometry = super::arc_geometry(preview, -240.0, 60.0);

        assert!((preview - 0.502).abs() < f32::EPSILON);
        assert!((geometry.active_sweep_deg - 150.6).abs() < 0.0001);
        assert!((geometry.tick_angle_deg - -89.4).abs() < 0.0001);
    }
}
