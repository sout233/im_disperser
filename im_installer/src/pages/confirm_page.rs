use vizia::{prelude::*, vg::Pixel};

use crate::AppEvent;

#[derive(Lens)]
pub(crate) struct ConfirmPage {}

pub(crate) enum ConfirmPageEvent {}

impl ConfirmPage {
    pub fn new(
        cx: &mut Context,
        is_install_vst3: impl Lens<Target = bool>,
        is_install_clap: impl Lens<Target = bool>,
        vst3_path: impl Lens<Target = String>,
        clap_path: impl Lens<Target = String>,
    ) -> Handle<'_, Self> {
        ConfirmPage {}.build(cx, |cx| {
            VStack::new(cx, |cx| {
                Label::new(cx, Localized::new("confirm-install")).class("p");

                Binding::new(cx, is_install_vst3, move |cx, is| {
                    if is.get(cx) {
                        HStack::new(cx, |cx| {
                            Label::new(cx, "VST3")
                                .class("p-xs")
                                .width(Pixels(50.0))
                                .font_weight(FontWeightKeyword::Bold);
                            Label::new(cx, vst3_path).class("p-xs");
                        })
                        .class("textbox-stack");
                    }
                });

                Binding::new(cx, is_install_clap, move |cx, is| {
                    if is.get(cx) {
                        HStack::new(cx, |cx| {
                            Label::new(cx, "CLAP")
                                .class("p-xs")
                                .width(Pixels(50.0))
                                .font_weight(FontWeightKeyword::Bold);
                            Label::new(cx, clap_path).class("p-xs");
                        })
                        .class("textbox-stack");
                    }
                });
            })
            .class("opt-panel");
        })
    }
}

impl View for ConfirmPage {
    fn element(&self) -> Option<&'static str> {
        Some("select-format-page")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        // event.map(|app_event, meta| match app_event {
        //     ConfirmPageEvent::Install => {
        //         self.show_install_btn = false;
        //         self.subtitle = "正在安装...".to_string();
        //     }
        // });
    }
}
