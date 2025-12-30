use vizia::prelude::Data;

#[derive(Data, PartialEq, Clone, Copy)]
pub enum Language {
    SimplifiedChinese,
    English,
}

impl ToString for Language {
    fn to_string(&self) -> String {
        match self {
            Language::SimplifiedChinese => "简体中文".to_string(),
            Language::English => "English".to_string(),
        }
    }
}

pub(crate) trait ToLocalizeKey {
    fn to_localize_key(&self) -> String;
}

impl ToLocalizeKey for Language {
    fn to_localize_key(&self) -> String {
        match self {
            Language::SimplifiedChinese => "zh-CN".to_string(),
            Language::English => "en-US".to_string(),
        }
    }
}
