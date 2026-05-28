use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LocaleKey(pub String);

impl LocaleKey {
    pub fn new(key: impl Into<String>) -> Self { Self(key.into()) }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LocaleCode(pub String);

impl LocaleCode {
    pub fn new(code: impl Into<String>) -> Self { Self(code.into()) }
    pub fn label(&self) -> &str { &self.0 }
}

#[derive(Clone, Debug)]
pub struct LocaleEntry {
    pub key: String,
    pub translations: HashMap<String, String>,
    pub comment: String,
}

impl LocaleEntry {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into(), translations: HashMap::new(), comment: String::new() }
    }

    pub fn set(&mut self, locale: impl Into<String>, text: impl Into<String>) {
        self.translations.insert(locale.into(), text.into());
    }

    pub fn get(&self, locale: &str) -> Option<&str> {
        self.translations.get(locale).map(|s| s.as_str())
    }
}

#[derive(Clone, Debug)]
pub struct LocalizationTable {
    pub name: String,
    pub locales: Vec<String>,
    pub entries: Vec<LocaleEntry>,
    pub fallback_locale: String,
    pub active_locale: String,
}

impl Default for LocalizationTable {
    fn default() -> Self {
        let mut t = Self {
            name: "Game Strings".to_string(),
            locales: vec!["fr".to_string(), "en".to_string(), "es".to_string(), "de".to_string(), "ja".to_string()],
            entries: Vec::new(),
            fallback_locale: "en".to_string(),
            active_locale: "fr".to_string(),
        };
        let mut e0 = LocaleEntry::new("ui.start_game");
        e0.set("fr", "Nouvelle partie");
        e0.set("en", "New Game");
        e0.set("es", "Nuevo juego");
        e0.set("de", "Neues Spiel");
        e0.set("ja", "新しいゲーム");
        t.entries.push(e0);
        let mut e1 = LocaleEntry::new("ui.continue");
        e1.set("fr", "Continuer");
        e1.set("en", "Continue");
        e1.set("es", "Continuar");
        e1.set("de", "Fortfahren");
        e1.set("ja", "続ける");
        t.entries.push(e1);
        let mut e2 = LocaleEntry::new("ui.settings");
        e2.set("fr", "Paramètres");
        e2.set("en", "Settings");
        e2.set("es", "Configuración");
        e2.set("de", "Einstellungen");
        e2.set("ja", "設定");
        t.entries.push(e2);
        let mut e3 = LocaleEntry::new("ui.quit");
        e3.set("fr", "Quitter");
        e3.set("en", "Quit");
        e3.set("es", "Salir");
        e3.set("de", "Beenden");
        e3.set("ja", "終了");
        t.entries.push(e3);
        t
    }
}

impl LocalizationTable {
    pub fn new() -> Self { Self::default() }

    pub fn tr<'a>(&'a self, key: &'a str) -> &'a str {
        for entry in &self.entries {
            if entry.key == key {
                if let Some(t) = entry.get(&self.active_locale) { return t; }
                if let Some(t) = entry.get(&self.fallback_locale) { return t; }
                return &entry.key;
            }
        }
        key
    }

    pub fn add_entry(&mut self, key: impl Into<String>) -> usize {
        let idx = self.entries.len();
        self.entries.push(LocaleEntry::new(key));
        idx
    }

    pub fn remove_entry(&mut self, index: usize) {
        if index < self.entries.len() { self.entries.remove(index); }
    }

    pub fn add_locale(&mut self, code: impl Into<String>) {
        let c = code.into();
        if !self.locales.contains(&c) { self.locales.push(c); }
    }

    pub fn missing_translations(&self) -> Vec<(&str, &str)> {
        let mut result = Vec::new();
        for entry in &self.entries {
            for locale in &self.locales {
                if !entry.translations.contains_key(locale.as_str()) {
                    result.push((entry.key.as_str(), locale.as_str()));
                }
            }
        }
        result
    }
}
