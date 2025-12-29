use vizia::prelude::*;

#[derive(Lens)]
pub struct AppData {
    install_vst3: bool,
    install_clap: bool,
}

pub enum AppEvent {
    ToggleInstallVst3,
    ToggleInstallClap,
}

impl Model for AppData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, meta| match app_event {
            AppEvent::ToggleInstallVst3 => self.install_vst3 = !self.install_vst3,
            AppEvent::ToggleInstallClap => self.install_clap = !self.install_clap,
        });
    }
}

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        cx.add_stylesheet(include_style!("src/style.css"))
            .expect("unable to load style.css");

        cx.add_font_mem(include_bytes!("../../assets/JetBrainsMono-Bold.ttf"));

        AppData {
            install_vst3: false,
            install_clap: false,
        }
        .build(cx);

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                Label::new(cx, "IM_DISPERSER").class("title");
                Label::new(cx, "一个一个一个 Disperser 插件").class("subtitle");
                VStack::new(cx, |cx| {
                    Label::new(cx, "选择要安装的格式：").class("p");

                    HStack::new(cx, |cx| {
                        Checkbox::new(cx, AppData::install_vst3)
                            .on_toggle(|ex| {
                                ex.emit(AppEvent::ToggleInstallVst3);
                            })
                            .class("checkbox");
                        Label::new(cx, "VST3");
                    })
                    .class("checkbox-stack");

                    HStack::new(cx, |cx| {
                        Checkbox::new(cx, AppData::install_clap)
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
            .width(Stretch(1.0));

            VStack::new(cx, |cx| {
                Button::new(cx, |cx| Label::new(cx, "下一步")).class("next-btn");
            })
            .alignment(Alignment::BottomRight)
            .width(Stretch(1.0));
        })
        .class("main-stack");
    })
    .inner_size((800, 333))
    .title("IM_DISPERSER INSTALLER")
    .anchor_target(AnchorTarget::Monitor)
    .parent_anchor(Anchor::Center)
    .enabled_window_buttons(WindowButtons::empty())
    .enabled_window_buttons(WindowButtons::MINIMIZE)
    .enabled_window_buttons(WindowButtons::CLOSE)
    .run()
}
