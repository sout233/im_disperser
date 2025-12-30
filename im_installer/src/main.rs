use std::{
    fs::{self, write},
    panic,
    path::Path,
    process,
    str::FromStr,
    thread,
};

use vizia::{prelude::*, vg::Pixel};

use crate::{
    pages::{
        confirm_page::ConfirmPage,
        done_page::DonePage,
        installing_page::{InstallingPage, InstallingPageEvent},
        select_format_page::SelectFormatPage,
        select_path_page::SelectPathPage,
    },
    utils::localize::{Language, ToLocalizeKey},
};

mod pages;
mod utils;

#[derive(Lens)]
pub struct AppData {
    install_vst3: bool,
    install_clap: bool,
    current_page: TabPage,
    show_prev_btn: bool,
    show_next_btn: bool,
    disable_next_btn: bool,
    vst3_path: String,
    clap_path: String,
    next_btn_text: Localized,
    lang: Language,
}

pub enum AppEvent {
    ToggleInstallVst3,
    ToggleInstallClap,
    PrevPage,
    NextPage,
    UpdateVst3Path(String),
    UpdateClapPath(String),
    StartInstallation,
    UpdateButtons,
    UpdateLanguage(Language),
}

#[derive(Data, PartialEq, Clone)]
pub enum TabPage {
    SelectFormat,
    SelectPath,
    Confirm,
    Installing,
    Done,
}

impl Model for AppData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, meta| match app_event {
            AppEvent::ToggleInstallVst3 => {
                self.install_vst3 = !self.install_vst3;
                if !self.install_clap && !self.install_vst3 {
                    self.disable_next_btn = true;
                } else {
                    self.disable_next_btn = false;
                }
            }
            AppEvent::ToggleInstallClap => {
                self.install_clap = !self.install_clap;
                if !self.install_clap && !self.install_vst3 {
                    self.disable_next_btn = true;
                } else {
                    self.disable_next_btn = false;
                }
            }
            AppEvent::PrevPage => {
                match self.current_page {
                    TabPage::SelectFormat => self.current_page = TabPage::SelectFormat,
                    TabPage::SelectPath => self.current_page = TabPage::SelectFormat,
                    TabPage::Confirm => self.current_page = TabPage::SelectPath,
                    TabPage::Installing => panic!("这不可能！"),
                    TabPage::Done=> panic!("这不可能！")
                }

                cx.emit(AppEvent::UpdateButtons);
            }
            AppEvent::NextPage => {
                match self.current_page {
                    TabPage::SelectFormat => self.current_page = TabPage::SelectPath,
                    TabPage::SelectPath => self.current_page = TabPage::Confirm,
                    TabPage::Confirm => {
                        self.current_page = TabPage::Installing;
                        cx.emit(AppEvent::StartInstallation);
                    }
                    TabPage::Installing => self.current_page = TabPage::Done,
                    TabPage::Done => {
                        process::exit(0);
                    },
                }

                cx.emit(AppEvent::UpdateButtons);
            }
            AppEvent::UpdateVst3Path(path) => self.vst3_path = path.clone(),
            AppEvent::UpdateClapPath(path) => self.clap_path = path.clone(),
            AppEvent::StartInstallation => {
                if self.install_vst3 {
                    const VST3_FILE_DATA: &[u8] = include_bytes!(
                        "../../target/bundled/im_disperser.vst3/Contents/x86_64-win/im_disperser.vst3"
                    );

                    let vst3_target_path = Path::new(&self.vst3_path)
                        .join("im_disperser")
                        .join("Contents")
                        .join("im_disperser.vst3");

                    if let Some(parent) = vst3_target_path.parent() {
                        ok_or_msgbox!(fs::create_dir_all(parent), "Failed to create VST3 directory");
                    }

                    ok_or_msgbox!(
                        fs::write(&vst3_target_path, VST3_FILE_DATA),
                        "Failed to write VST3 file (Permission denied?)"
                    );

                    println!("Installed VST3: {:?}", vst3_target_path);
                }

                if self.install_clap {
                    println!("Installing CLAP...");
                    const CLAP_FILE_DATA: &[u8] = include_bytes!(
                        "../../target/bundled/im_disperser.clap"
                    );

                    let clap_target_path = Path::new(&self.clap_path)
                        .join("im_disperser")
                        .join("im_disperser.clap");

                    if let Some(parent) = clap_target_path.parent() {
                        ok_or_msgbox!(fs::create_dir_all(parent), "Failed to create CLAP directory");
                    }

                    ok_or_msgbox!(
                        fs::write(&clap_target_path, CLAP_FILE_DATA),
                        "Failed to write CLAP file"
                    );

                    println!("Installed CLAP: {:?}", clap_target_path);
                }

                cx.emit(AppEvent::NextPage);
            }
            AppEvent::UpdateButtons => {
                let next_btn_blacklist = vec![
                    TabPage::Installing,
                ];

                let prev_btn_blacklist = vec![
                    TabPage::SelectFormat,
                    TabPage::Installing,
                    TabPage::Done
                ];

                let next_btn_enabled = !next_btn_blacklist.contains(&self.current_page);
                let prev_btn_enabled = !prev_btn_blacklist.contains(&self.current_page);

                self.show_next_btn = next_btn_enabled;
                self.show_prev_btn = prev_btn_enabled;

                if self.current_page == TabPage::Confirm {
                    self.next_btn_text = Localized::new("install");
                }
               else if self.current_page == TabPage::Done {
                    self.next_btn_text = Localized::new("done");
                }
                else {
                    self.next_btn_text = Localized::new("next");
                }
            },
            AppEvent::UpdateLanguage(language) => {
                self.lang = language.clone();
                let id = LanguageIdentifier::from_str(language.to_localize_key().as_str()).unwrap();
                cx.emit(EnvironmentEvent::SetLocale(id.clone()));
                self.next_btn_text = Localized::new("done");
                cx.emit(AppEvent::UpdateButtons);
                println!("set language to {}", id.to_string());
            },
        });
    }
}

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        cx.add_stylesheet(include_style!("src/style.css"))
            .expect("unable to load style.css");

        cx.add_font_mem(include_bytes!("../../assets/JetBrainsMono-Bold.ttf"));

        cx.add_translation(
            langid!("en-US"),
            include_str!("resources/en-US/main.flt").to_owned(),
        );

        cx.add_translation(
            langid!("zh-CN"),
            include_str!("resources/zh-CN/main.flt").to_owned(),
        );

        AppData {
            install_vst3: false,
            install_clap: false,
            current_page: TabPage::SelectFormat,
            show_prev_btn: false,
            show_next_btn: true,
            disable_next_btn: true,
            vst3_path: String::from(r#"C:\Program Files\Common Files\VST3"#),
            clap_path: String::from(r#"C:\Program Files\Common Files\CLAP"#),
            next_btn_text: Localized::new("next"),
            lang: Language::English,
        }
        .build(cx);

        cx.emit(AppEvent::UpdateLanguage(Language::English));

        let is_elevated = ok_or_msgbox!(
            windows_elevate::check_elevated(),
            "Failed to call check_elevated"
        );

        if !is_elevated {
            let exe_path = std::env::current_exe().expect("Failed to get current exe path");

            let cmd = privesc::PrivilegedCommand::new(&exe_path)
                .args(std::env::args().skip(1))
                .gui(true);

            match cmd.spawn() {
                Ok(_) => {
                    std::process::exit(0);
                }
                Err(e) => {
                    ok_or_msgbox!(
                        Err(e),
                        "User rejected privilege request, installation cannot continue"
                    );
                    process::exit(0);
                }
            }
        }

        HStack::new(cx, |cx| {
            HStack::new(cx, |_| {}).class("shadow-bar");

            VStack::new(cx, |cx| {
                Label::new(cx, "IM_DISPERSER").class("title");
                Label::new(cx, Localized::new("subtitle")).class("subtitle");

                Binding::new(cx, AppData::current_page, |cx, current_page| {
                    match current_page.get(cx) {
                        TabPage::SelectFormat => {
                            SelectFormatPage::new(cx, AppData::install_vst3, AppData::install_clap);
                        }
                        TabPage::SelectPath => {
                            SelectPathPage::new(cx, AppData::install_vst3, AppData::install_clap);
                        }
                        TabPage::Confirm => {
                            ConfirmPage::new(
                                cx,
                                AppData::install_vst3,
                                AppData::install_clap,
                                AppData::vst3_path,
                                AppData::clap_path,
                            );
                        }
                        TabPage::Installing => {
                            InstallingPage::new(cx);
                        }
                        TabPage::Done => {
                            DonePage::new(cx);
                        }
                    };
                });
            })
            .width(Stretch(1.0));

            VStack::new(cx, |cx| {
                Binding::new(cx, AppData::lang, |cx, lang| {
                    let lang = lang.get(cx);
                    HStack::new(cx, |cx| {
                        Button::new(cx, |cx| Label::new(cx, lang.to_string()))
                            .on_press(move |ex| {
                                if lang == Language::SimplifiedChinese {
                                    ex.emit(AppEvent::UpdateLanguage(Language::English));
                                } else {
                                    ex.emit(AppEvent::UpdateLanguage(Language::SimplifiedChinese));
                                }
                            })
                            .class("btn");
                    })
                    .alignment(Alignment::TopRight)
                    .width(Stretch(1.0))
                    .height(Stretch(1.0));
                });

                HStack::new(cx, |cx| {
                    Binding::new(cx, AppData::show_prev_btn, |cx, show| {
                        if show.get(cx) {
                            Button::new(cx, |cx| Label::new(cx, Localized::new("prev")))
                                .on_press(|ex| {
                                    ex.emit(AppEvent::PrevPage);
                                })
                                .visibility(AppData::show_prev_btn)
                                .class("next-btn");
                        }
                    });

                    Binding::new(cx, AppData::show_next_btn, |cx, show| {
                        if show.get(cx) {
                            Button::new(cx, |cx| Label::new(cx, AppData::next_btn_text))
                                .on_press(|ex| {
                                    ex.emit(AppEvent::NextPage);
                                })
                                .disabled(AppData::disable_next_btn)
                                .class("next-btn");
                        }
                    });
                })
                .gap(Pixels(4.0))
                .alignment(Alignment::BottomRight)
                .width(Stretch(1.0));
            })
            .width(Stretch(1.0))
            .height(Stretch(1.0));
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
