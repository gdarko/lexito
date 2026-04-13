#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locale {
    pub code: &'static str,
    pub name: &'static str,
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} \u{2014} {}", self.code, self.name)
    }
}

pub fn all_locales() -> Vec<Locale> {
    ALL_LOCALES.to_vec()
}

pub const ALL_LOCALES: &[Locale] = &[
    Locale {
        code: "af",
        name: "Afrikaans",
    },
    Locale {
        code: "ak",
        name: "Akan",
    },
    Locale {
        code: "am",
        name: "Amharic",
    },
    Locale {
        code: "an",
        name: "Aragonese",
    },
    Locale {
        code: "ar",
        name: "Arabic",
    },
    Locale {
        code: "ar_DZ",
        name: "Arabic (Algeria)",
    },
    Locale {
        code: "ar_EG",
        name: "Arabic (Egypt)",
    },
    Locale {
        code: "ar_MA",
        name: "Arabic (Morocco)",
    },
    Locale {
        code: "ar_SA",
        name: "Arabic (Saudi Arabia)",
    },
    Locale {
        code: "as",
        name: "Assamese",
    },
    Locale {
        code: "ast",
        name: "Asturian",
    },
    Locale {
        code: "az",
        name: "Azerbaijani",
    },
    Locale {
        code: "be",
        name: "Belarusian",
    },
    Locale {
        code: "bg",
        name: "Bulgarian",
    },
    Locale {
        code: "bn",
        name: "Bengali",
    },
    Locale {
        code: "bn_BD",
        name: "Bengali (Bangladesh)",
    },
    Locale {
        code: "bn_IN",
        name: "Bengali (India)",
    },
    Locale {
        code: "bo",
        name: "Tibetan",
    },
    Locale {
        code: "br",
        name: "Breton",
    },
    Locale {
        code: "bs",
        name: "Bosnian",
    },
    Locale {
        code: "ca",
        name: "Catalan",
    },
    Locale {
        code: "ce",
        name: "Chechen",
    },
    Locale {
        code: "ckb",
        name: "Central Kurdish",
    },
    Locale {
        code: "co",
        name: "Corsican",
    },
    Locale {
        code: "cs",
        name: "Czech",
    },
    Locale {
        code: "cy",
        name: "Welsh",
    },
    Locale {
        code: "da",
        name: "Danish",
    },
    Locale {
        code: "de",
        name: "German",
    },
    Locale {
        code: "de_AT",
        name: "German (Austria)",
    },
    Locale {
        code: "de_CH",
        name: "German (Switzerland)",
    },
    Locale {
        code: "dz",
        name: "Dzongkha",
    },
    Locale {
        code: "el",
        name: "Greek",
    },
    Locale {
        code: "en",
        name: "English",
    },
    Locale {
        code: "en_AU",
        name: "English (Australia)",
    },
    Locale {
        code: "en_CA",
        name: "English (Canada)",
    },
    Locale {
        code: "en_GB",
        name: "English (United Kingdom)",
    },
    Locale {
        code: "en_NZ",
        name: "English (New Zealand)",
    },
    Locale {
        code: "en_ZA",
        name: "English (South Africa)",
    },
    Locale {
        code: "eo",
        name: "Esperanto",
    },
    Locale {
        code: "es",
        name: "Spanish",
    },
    Locale {
        code: "es_AR",
        name: "Spanish (Argentina)",
    },
    Locale {
        code: "es_CL",
        name: "Spanish (Chile)",
    },
    Locale {
        code: "es_CO",
        name: "Spanish (Colombia)",
    },
    Locale {
        code: "es_CR",
        name: "Spanish (Costa Rica)",
    },
    Locale {
        code: "es_EC",
        name: "Spanish (Ecuador)",
    },
    Locale {
        code: "es_GT",
        name: "Spanish (Guatemala)",
    },
    Locale {
        code: "es_HN",
        name: "Spanish (Honduras)",
    },
    Locale {
        code: "es_MX",
        name: "Spanish (Mexico)",
    },
    Locale {
        code: "es_PE",
        name: "Spanish (Peru)",
    },
    Locale {
        code: "es_PR",
        name: "Spanish (Puerto Rico)",
    },
    Locale {
        code: "es_UY",
        name: "Spanish (Uruguay)",
    },
    Locale {
        code: "es_VE",
        name: "Spanish (Venezuela)",
    },
    Locale {
        code: "et",
        name: "Estonian",
    },
    Locale {
        code: "eu",
        name: "Basque",
    },
    Locale {
        code: "fa",
        name: "Persian",
    },
    Locale {
        code: "ff",
        name: "Fulah",
    },
    Locale {
        code: "fi",
        name: "Finnish",
    },
    Locale {
        code: "fil",
        name: "Filipino",
    },
    Locale {
        code: "fo",
        name: "Faroese",
    },
    Locale {
        code: "fr",
        name: "French",
    },
    Locale {
        code: "fr_BE",
        name: "French (Belgium)",
    },
    Locale {
        code: "fr_CA",
        name: "French (Canada)",
    },
    Locale {
        code: "fr_CH",
        name: "French (Switzerland)",
    },
    Locale {
        code: "fy",
        name: "Western Frisian",
    },
    Locale {
        code: "ga",
        name: "Irish",
    },
    Locale {
        code: "gd",
        name: "Scottish Gaelic",
    },
    Locale {
        code: "gl",
        name: "Galician",
    },
    Locale {
        code: "gu",
        name: "Gujarati",
    },
    Locale {
        code: "gv",
        name: "Manx",
    },
    Locale {
        code: "ha",
        name: "Hausa",
    },
    Locale {
        code: "he",
        name: "Hebrew",
    },
    Locale {
        code: "hi",
        name: "Hindi",
    },
    Locale {
        code: "hr",
        name: "Croatian",
    },
    Locale {
        code: "ht",
        name: "Haitian Creole",
    },
    Locale {
        code: "hu",
        name: "Hungarian",
    },
    Locale {
        code: "hy",
        name: "Armenian",
    },
    Locale {
        code: "ia",
        name: "Interlingua",
    },
    Locale {
        code: "id",
        name: "Indonesian",
    },
    Locale {
        code: "ig",
        name: "Igbo",
    },
    Locale {
        code: "is",
        name: "Icelandic",
    },
    Locale {
        code: "it",
        name: "Italian",
    },
    Locale {
        code: "ja",
        name: "Japanese",
    },
    Locale {
        code: "jv",
        name: "Javanese",
    },
    Locale {
        code: "ka",
        name: "Georgian",
    },
    Locale {
        code: "kab",
        name: "Kabyle",
    },
    Locale {
        code: "kk",
        name: "Kazakh",
    },
    Locale {
        code: "km",
        name: "Khmer",
    },
    Locale {
        code: "kn",
        name: "Kannada",
    },
    Locale {
        code: "ko",
        name: "Korean",
    },
    Locale {
        code: "ku",
        name: "Kurdish",
    },
    Locale {
        code: "ky",
        name: "Kyrgyz",
    },
    Locale {
        code: "la",
        name: "Latin",
    },
    Locale {
        code: "lb",
        name: "Luxembourgish",
    },
    Locale {
        code: "lo",
        name: "Lao",
    },
    Locale {
        code: "lt",
        name: "Lithuanian",
    },
    Locale {
        code: "lv",
        name: "Latvian",
    },
    Locale {
        code: "mg",
        name: "Malagasy",
    },
    Locale {
        code: "mi",
        name: "Maori",
    },
    Locale {
        code: "mk",
        name: "Macedonian",
    },
    Locale {
        code: "ml",
        name: "Malayalam",
    },
    Locale {
        code: "mn",
        name: "Mongolian",
    },
    Locale {
        code: "mr",
        name: "Marathi",
    },
    Locale {
        code: "ms",
        name: "Malay",
    },
    Locale {
        code: "mt",
        name: "Maltese",
    },
    Locale {
        code: "my",
        name: "Burmese",
    },
    Locale {
        code: "nb",
        name: "Norwegian Bokm\u{e5}l",
    },
    Locale {
        code: "ne",
        name: "Nepali",
    },
    Locale {
        code: "nl",
        name: "Dutch",
    },
    Locale {
        code: "nl_BE",
        name: "Dutch (Belgium)",
    },
    Locale {
        code: "nn",
        name: "Norwegian Nynorsk",
    },
    Locale {
        code: "oc",
        name: "Occitan",
    },
    Locale {
        code: "or",
        name: "Odia",
    },
    Locale {
        code: "pa",
        name: "Punjabi",
    },
    Locale {
        code: "pl",
        name: "Polish",
    },
    Locale {
        code: "ps",
        name: "Pashto",
    },
    Locale {
        code: "pt",
        name: "Portuguese",
    },
    Locale {
        code: "pt_BR",
        name: "Portuguese (Brazil)",
    },
    Locale {
        code: "pt_PT",
        name: "Portuguese (Portugal)",
    },
    Locale {
        code: "rm",
        name: "Romansh",
    },
    Locale {
        code: "ro",
        name: "Romanian",
    },
    Locale {
        code: "ru",
        name: "Russian",
    },
    Locale {
        code: "rw",
        name: "Kinyarwanda",
    },
    Locale {
        code: "sa",
        name: "Sanskrit",
    },
    Locale {
        code: "sc",
        name: "Sardinian",
    },
    Locale {
        code: "sd",
        name: "Sindhi",
    },
    Locale {
        code: "si",
        name: "Sinhala",
    },
    Locale {
        code: "sk",
        name: "Slovak",
    },
    Locale {
        code: "sl",
        name: "Slovenian",
    },
    Locale {
        code: "so",
        name: "Somali",
    },
    Locale {
        code: "sq",
        name: "Albanian",
    },
    Locale {
        code: "sr",
        name: "Serbian",
    },
    Locale {
        code: "sr_Latn",
        name: "Serbian (Latin)",
    },
    Locale {
        code: "su",
        name: "Sundanese",
    },
    Locale {
        code: "sv",
        name: "Swedish",
    },
    Locale {
        code: "sw",
        name: "Swahili",
    },
    Locale {
        code: "ta",
        name: "Tamil",
    },
    Locale {
        code: "te",
        name: "Telugu",
    },
    Locale {
        code: "tg",
        name: "Tajik",
    },
    Locale {
        code: "th",
        name: "Thai",
    },
    Locale {
        code: "ti",
        name: "Tigrinya",
    },
    Locale {
        code: "tk",
        name: "Turkmen",
    },
    Locale {
        code: "tl",
        name: "Tagalog",
    },
    Locale {
        code: "tr",
        name: "Turkish",
    },
    Locale {
        code: "tt",
        name: "Tatar",
    },
    Locale {
        code: "ug",
        name: "Uyghur",
    },
    Locale {
        code: "uk",
        name: "Ukrainian",
    },
    Locale {
        code: "ur",
        name: "Urdu",
    },
    Locale {
        code: "uz",
        name: "Uzbek",
    },
    Locale {
        code: "vi",
        name: "Vietnamese",
    },
    Locale {
        code: "wa",
        name: "Walloon",
    },
    Locale {
        code: "wo",
        name: "Wolof",
    },
    Locale {
        code: "xh",
        name: "Xhosa",
    },
    Locale {
        code: "yi",
        name: "Yiddish",
    },
    Locale {
        code: "yo",
        name: "Yoruba",
    },
    Locale {
        code: "zh_CN",
        name: "Chinese (Simplified)",
    },
    Locale {
        code: "zh_HK",
        name: "Chinese (Hong Kong)",
    },
    Locale {
        code: "zh_TW",
        name: "Chinese (Traditional)",
    },
    Locale {
        code: "zu",
        name: "Zulu",
    },
];
