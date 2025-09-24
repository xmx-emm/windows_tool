/// Steam 所有的语言列表
/// 更新日期2025-10-8
/// https://partner.steamgames.com/doc/store/localization/languages
/// ```
/// use windows_tool::steam::language::SteamLanguage;
/// let w = SteamLanguage::from_language_code("schinese");
/// println!("w {:#?}",w)
/// ```
#[derive(Debug, Clone, Copy)]
#[derive(Eq, Hash, PartialEq)]
pub struct SteamLanguage {
    pub cn_name: &'static str,
    pub name: &'static str,
    pub language: &'static str,
    pub web_api_language: &'static str,
}

// 定义宏来简化创建语言常量
macro_rules! define_steam_languages {
    ($($const_name:ident: ($cn_name:expr, $name:expr, $language:expr, $web_api_language:expr)),* $(,)?) => {
        impl SteamLanguage {
            $(
                pub const $const_name: SteamLanguage = SteamLanguage {
                    cn_name: $cn_name,
                    name: $name,
                    language: $language,
                    web_api_language: $web_api_language,
                };
            )*

            // 获取所有支持的语言列表
            pub fn all_languages() -> Vec<&'static SteamLanguage> {
                vec![
                    $(&Self::$const_name),*
                ]
            }

            // 根据语言代码查找语言
            pub fn from_language_code(language_code: &str) -> Option<&'static SteamLanguage> {
                Self::all_languages().into_iter().find(|lang| lang.language == language_code)
            }

            // 根据Web API语言代码查找语言
            pub fn from_web_api_code(web_api_code: &str) -> Option<&'static SteamLanguage> {
                Self::all_languages().into_iter().find(|lang| lang.web_api_language == web_api_code)
            }
        }
    };
}

// 使用宏定义所有语言
define_steam_languages! {
    ARABIC: ("阿拉伯语 *", "العربية", "arabic", "ar"),
    BULGARIAN: ("保加利亚语", "български език", "bulgarian", "bg"),
    SIMPLIFIED_CHINESE: ("简体中文", "简体中文", "schinese", "zh-CN"),
    TRADITIONAL_CHINESE: ("繁体中文", "繁體中文", "tchinese", "zh-TW"),
    CZECH: ("捷克语", "čeština", "czech", "cs"),
    DANISH: ("丹麦语", "Dansk", "danish", "da"),
    DUTCH: ("荷兰语", "Nederlands", "dutch", "nl"),
    ENGLISH: ("英语", "English", "english", "en"),
    FINNISH: ("芬兰语", "Suomi", "finnish", "fi"),
    FRENCH: ("法语", "Français", "french", "fr"),
    GERMAN: ("德语", "Deutsch", "german", "de"),
    GREEK: ("希腊语", "Ελληνικά", "greek", "el"),
    HUNGARIAN: ("匈牙利语", "Magyar", "hungarian", "hu"),
    INDONESIAN: ("印度尼西亚语", "Bahasa Indonesia", "indonesian", "id"),
    ITALIAN: ("意大利语", "Italiano", "italian", "it"),
    JAPANESE: ("日语", "日本語", "japanese", "ja"),
    KOREAN: ("韩语", "한국어", "koreana", "ko"),
    NORWEGIAN: ("挪威语", "Norsk", "norwegian", "no"),
    POLISH: ("波兰语", "Polski", "polish", "pl"),
    PORTUGUESE: ("葡萄牙语", "Português", "portuguese", "pt"),
    BRAZILIAN_PORTUGUESE: ("葡萄牙语 - 巴西", "Português-Brasil", "brazilian", "pt-BR"),
    ROMANIAN: ("罗马尼亚语", "Română", "romanian", "ro"),
    RUSSIAN: ("俄语", "Русский", "russian", "ru"),
    SPANISH: ("西班牙语 - 西班牙", "Español-España", "spanish", "es"),
    LATIN_AMERICAN_SPANISH: ("西班牙语 - 拉丁美洲", "Español-Latinoamérica", "latam", "es-419"),
    SWEDISH: ("瑞典语", "Svenska", "swedish", "sv"),
    THAI: ("泰语", "ไทย", "thai", "th"),
    TURKISH: ("土耳其语", "Türkçe", "turkish", "tr"),
    UKRAINIAN: ("乌克兰语", "Українська", "ukrainian", "uk"),
    VIETNAMESE: ("越南语", "Tiếng Việt", "vietnamese", "vi"),
}
