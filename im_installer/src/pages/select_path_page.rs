use vizia::prelude::*;

use crate::{AppData, AppEvent};

pub(crate) struct SelectPathPage {}

impl SelectPathPage {
    pub fn new(
        cx: &mut Context,
        is_install_vst3: impl Lens<Target = bool>,
        is_install_clap: impl Lens<Target = bool>,
    ) -> Handle<'_, Self> {
        SelectPathPage {}.build(cx, |cx| {
            VStack::new(cx, |cx| {
                Label::new(cx, Localized::new("choose-path")).class("p");
                Label::new(cx, Localized::new("choose-path-info")).class("p-xs");

                HStack::new(cx, |cx| {
                    Label::new(cx, "VST3");
                    Textbox::new(cx, AppData::vst3_path)
                        .disabled(is_install_vst3.map(|is| !is))
                        .on_edit(|ex, txt| {
                            ex.emit(AppEvent::UpdateVst3Path(txt));
                        })
                        .class("textbox");
                    Button::new(cx, |cx| Label::new(cx, "..."))
                        .on_press(|ex| {
                            let dialog = rfd::FileDialog::new()
                                .set_title("Select VST3 Path")
                                .pick_folder();
                            if let Some(path) = dialog {
                                ex.emit(AppEvent::UpdateVst3Path(
                                    path.to_string_lossy().to_string(),
                                ));
                            }
                        })
                        .class("btn");
                })
                .class("textbox-stack");

                HStack::new(cx, |cx| {
                    Label::new(cx, "CLAP");
                    Textbox::new(cx, AppData::clap_path)
                        .on_edit(|ex, txt| {
                            ex.emit(AppEvent::UpdateClapPath(txt));
                        })
                        .disabled(is_install_clap.map(|is| !is))
                        .class("textbox");
                    Button::new(cx, |cx| Label::new(cx, "..."))
                        .on_press(|ex| {
                            let dialog = rfd::FileDialog::new()
                                .set_title("Select CLAP Path")
                                .pick_folder();
                            if let Some(path) = dialog {
                                ex.emit(AppEvent::UpdateClapPath(
                                    path.to_string_lossy().to_string(),
                                ));
                            }
                        })
                        .class("btn");
                })
                .class("textbox-stack");
            })
            .class("opt-panel");
        })
    }
}

impl View for SelectPathPage {
    fn element(&self) -> Option<&'static str> {
        Some("select-format-page")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {}
}
