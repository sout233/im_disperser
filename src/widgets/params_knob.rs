use nih_plug::prelude::Param;
use vizia_plug::vizia::prelude::*;
use vizia_plug::vizia::vg;
use vizia_plug::vizia::vg::Point;

use vizia_plug::widgets::param_base::ParamWidgetBase;
use vizia_plug::widgets::util::ModifiersExt;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;
use windows::Win32::UI::WindowsAndMessaging::ShowCursor;

static DEFAULT_DRAG_SCALAR: f32 = 0.0042;
static DEFAULT_WHEEL_SCALAR: f32 = 0.005;

#[derive(Debug, Clone, Copy, Default)]
pub struct DragStatus {
    drag_start_y: f32,
    drag_start_value: f32,
    drag_start_screen_pos: POINT,
}

#[derive(Lens)]
pub struct ParamKnob {
    param_base: ParamWidgetBase,
    text_input_active: bool,
    drag_status: Option<DragStatus>,
    drag_scalar: f32,
    wheel_scalar: f32,
    centered: bool,
}

enum ParamKnobEvent {
    CancelTextInput,
    TextInput(String),
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
        use self::ArcTrackHandle;

        Self {
            param_base: ParamWidgetBase::new(cx, params, params_to_param),
            text_input_active: false,
            drag_status: None,
            drag_scalar: DEFAULT_DRAG_SCALAR,
            wheel_scalar: DEFAULT_WHEEL_SCALAR,
            centered,
        }
        .build(
            cx,
            ParamWidgetBase::build_view(params, params_to_param, move |cx, param_data| {
                let normalized_value_lens =
                    param_data.make_lens(|param| param.unmodulated_normalized_value());
                let display_value_lens = param_data.make_lens(|param| {
                    param.normalized_value_to_string(param.unmodulated_normalized_value(), true)
                });

                let is_centered = centered;

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
                    } else {
                        ArcTrack::new(cx, is_centered, -150.0, 150.0).value(normalized_value_lens);
                    }
                });
            }),
        )
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

        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left)
            | WindowEvent::MouseTripleClick(MouseButton::Left) => {
                if cx.modifiers().alt() {
                    self.param_base.begin_set_parameter(cx);
                    self.param_base
                        .set_normalized_value(cx, self.param_base.default_normalized_value());
                    self.param_base.end_set_parameter(cx);
                    meta.consume();
                } else if cx.modifiers().command() {
                    self.text_input_active = true;
                    cx.set_active(true);
                    meta.consume();
                } else if !self.text_input_active {
                    cx.capture();
                    cx.focus();
                    cx.set_active(true);
                    force_hide_cursor();

                    let mut current_screen_pos = POINT::default();
                    unsafe {
                        GetCursorPos(&mut current_screen_pos).unwrap();
                    }

                    self.drag_status = Some(DragStatus {
                        drag_start_y: cx.mouse().cursor_y,
                        drag_start_value: self.param_base.unmodulated_normalized_value(),
                        drag_start_screen_pos: current_screen_pos,
                    });

                    self.param_base.begin_set_parameter(cx);
                    meta.consume();
                }
            }

            WindowEvent::MouseDoubleClick(MouseButton::Left)
            | WindowEvent::MouseDown(MouseButton::Right) => {
                self.param_base.begin_set_parameter(cx);
                self.param_base
                    .set_normalized_value(cx, self.param_base.default_normalized_value());
                self.param_base.end_set_parameter(cx);
                meta.consume();
            }

            WindowEvent::MouseUp(MouseButton::Left) => {
                if self.drag_status.is_some() {
                    self.drag_status = None;

                    // cx.set_cursor_icon(CursorIcon::Default);
                    // cx.set_cursor_grab(CursorGrabMode::Ungrab);
                    force_show_cursor();

                    cx.release();
                    cx.set_active(false);
                    self.param_base.end_set_parameter(cx);
                    meta.consume();
                }
            }

            WindowEvent::MouseMove(_, y) => {
                if let Some(status) = self.drag_status {
                    let mut current_screen_pos = POINT::default();
                    unsafe {
                        GetCursorPos(&mut current_screen_pos).unwrap();
                    }

                    let dy = status.drag_start_screen_pos.y - current_screen_pos.y;

                    if dy != 0 {
                        let mut value_delta = dy as f32 * self.drag_scalar;
                        if cx.modifiers().shift() {
                            value_delta *= 0.1;
                        }

                        let current_val = self.param_base.unmodulated_normalized_value();
                        let new_val = (current_val + value_delta).clamp(0.0, 1.0);
                        self.param_base.set_normalized_value(cx, new_val);

                        // 强行拉回
                        unsafe {
                            SetCursorPos(
                                status.drag_start_screen_pos.x,
                                status.drag_start_screen_pos.y,
                            )
                            .unwrap();
                        }
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
                    meta.consume();
                }
            }
            _ => {}
        });
    }
}

pub enum ArcTrackEvent {
    SetValue(f32),
}

pub struct ArcTrack {
    angle_start: f32,
    angle_end: f32,
    normalized_value: f32,
    center: bool,
}

impl ArcTrack {
    pub fn new(
        cx: &mut Context,
        center: bool,
        angle_start: f32,
        angle_end: f32,
    ) -> Handle<'_, Self> {
        Self {
            angle_start,
            angle_end,
            normalized_value: 0.0,
            center,
        }
        .build(cx, |_| {})
    }
}

pub trait ArcTrackHandle {
    fn value<L: Lens<Target = f32>>(self, lens: L) -> Self;
}

impl ArcTrackHandle for Handle<'_, ArcTrack> {
    fn value<L: Lens<Target = f32>>(mut self, lens: L) -> Self {
        let entity = self.entity();
        Binding::new(self.context(), lens, move |cx, value| {
            cx.emit_to(entity, ArcTrackEvent::SetValue(value.get(cx)));
        });
        self
    }
}

impl View for ArcTrack {
    fn element(&self) -> Option<&'static str> {
        Some("arctrack")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e: &ArcTrackEvent, _| match e {
            ArcTrackEvent::SetValue(val) => {
                self.normalized_value = *val;
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
        let start_angle_deg = self.angle_start + angle_offset;
        let end_angle_deg = self.angle_end + angle_offset;
        let sweep_angle_deg = end_angle_deg - start_angle_deg;

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

        // draw background track
        let mut paint = vg::Paint::default();
        paint.set_color(background_color);
        paint.set_stroke_width(stroke_width);
        paint.set_stroke_cap(vg::PaintCap::Butt);
        paint.set_style(vg::PaintStyle::Stroke);
        paint.set_anti_alias(true);
        canvas.draw_arc(&oval, start_angle_deg, sweep_angle_deg, false, &paint);

        // draw active arc
        let mut paint_fg = vg::Paint::default();
        paint_fg.set_color(foreground_color);
        paint_fg.set_stroke_width(stroke_width);
        paint_fg.set_stroke_cap(vg::PaintCap::Butt);
        paint_fg.set_style(vg::PaintStyle::Stroke);
        paint_fg.set_anti_alias(true);

        let value = self.normalized_value;

        if self.center {
            // let mid_angle_deg = start_angle_deg + sweep_angle_deg / 2.0;
            // let current_angle_deg = start_angle_deg + value * sweep_angle_deg;
            // let (draw_start, draw_sweep) = if value < 0.5 {
            //     (current_angle_deg, mid_angle_deg - current_angle_deg)
            // } else {
            //     (mid_angle_deg, current_angle_deg - mid_angle_deg)
            // };
            // canvas.draw_arc(&oval, draw_start, draw_sweep, false, &paint_fg);
            let current_sweep_deg = value * sweep_angle_deg;
            canvas.draw_arc(&oval, start_angle_deg, current_sweep_deg, false, &paint_fg);
        } else {
            let current_sweep_deg = value * sweep_angle_deg;
            canvas.draw_arc(&oval, start_angle_deg, current_sweep_deg, false, &paint_fg);
        }

        // draw indicator tick
        let current_angle_deg = start_angle_deg + value * sweep_angle_deg;
        let current_angle_rad = current_angle_deg.to_radians();

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

fn force_show_cursor() {
    while unsafe { ShowCursor(true) } < 0 {}
}

fn force_hide_cursor() {
    while unsafe { ShowCursor(false) } > 0 {}
}
