use vizia::prelude::*;

use crate::AppEvent;

#[derive(Lens)]
pub(crate) struct SelectFormatPage {}

impl SelectFormatPage {
    pub fn new(
        cx: &mut Context,
        select_vst3: impl Lens<Target = bool>,
        select_clap: impl Lens<Target = bool>,
    ) -> Handle<'_, Self> {
        SelectFormatPage {}.build(cx, |cx| {
            VStack::new(cx, |cx| {
                Label::new(cx, Localized::new("choose-format")).class("p");

                HStack::new(cx, |cx| {
                    Checkbox::new(cx, select_vst3)
                        .on_toggle(|ex| {
                            ex.emit(AppEvent::ToggleInstallVst3);
                        })
                        .class("checkbox");
                    Label::new(cx, "VST3");
                })
                .class("checkbox-stack");

                HStack::new(cx, |cx| {
                    Checkbox::new(cx, select_clap)
                        .on_toggle(|ex| {
                            ex.emit(AppEvent::ToggleInstallClap);
                        })
                        .class("checkbox");
                    Label::new(cx, "CLAP");
                })
                .class("checkbox-stack");
            })
            .class("opt-panel");
        })
    }
}

impl View for SelectFormatPage {
    fn element(&self) -> Option<&'static str> {
        Some("select-format-page")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        // event.map(|app_event, meta| match app_event {
        //     AppEvent::ToggleInstallVst3 => self.install_vst3 = !self.install_vst3,
        //     AppEvent::ToggleInstallClap => self.install_clap = !self.install_clap,
        // });
    }
}
