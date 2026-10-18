//! Lecture / écriture INI qui préserve l'ordre des sections et des clés (input.ini, game.cfg).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct IniSection {
    pub name: String,
    pub entries: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct IniDoc {
    pub sections: Vec<IniSection>,
}

impl IniDoc {
    pub fn parse(text: &str) -> Self {
        let mut doc = IniDoc::default();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') && !line.contains('=') {
                doc.sections.push(IniSection {
                    name: line[1..line.len() - 1].to_string(),
                    entries: vec![],
                });
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                if doc.sections.is_empty() {
                    doc.sections.push(IniSection::default());
                }
                doc.sections
                    .last_mut()
                    .unwrap()
                    .entries
                    .push((k.trim().to_string(), v.trim().to_string()));
            }
        }
        doc
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for s in &self.sections {
            if !s.name.is_empty() {
                out.push_str(&format!("[{}]\r\n", s.name));
            }
            for (k, v) in &s.entries {
                out.push_str(&format!("{k}={v}\r\n"));
            }
            out.push_str("\r\n");
        }
        out
    }

    pub fn section(&self, name: &str) -> Option<&IniSection> {
        self.sections
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name))
    }

    /// Remplace (ou ajoute) les sections données par celles de `other`.
    pub fn merge_sections(&mut self, other: &IniDoc, names: &[&str]) {
        for name in names {
            let Some(src) = other.section(name) else {
                continue;
            };
            match self
                .sections
                .iter_mut()
                .find(|s| s.name.eq_ignore_ascii_case(name))
            {
                Some(dst) => {
                    // On garde les clés locales inconnues du profil, on écrase celles du profil.
                    for (k, v) in &src.entries {
                        match dst.entries.iter_mut().find(|(dk, _)| dk == k) {
                            Some(e) => e.1 = v.clone(),
                            None => dst.entries.push((k.clone(), v.clone())),
                        }
                    }
                }
                None => self.sections.push(src.clone()),
            }
        }
    }

    pub fn only(&self, names: &[&str]) -> IniDoc {
        IniDoc {
            sections: self
                .sections
                .iter()
                .filter(|s| names.iter().any(|n| s.name.eq_ignore_ascii_case(n)))
                .cloned()
                .collect(),
        }
    }

    /// Depuis la réponse JSON du client LoL (`{Section: {cle: valeur}}`).
    pub fn from_json(v: &Value) -> Self {
        let mut doc = IniDoc::default();
        if let Some(obj) = v.as_object() {
            for (name, entries) in obj {
                let Some(e) = entries.as_object() else {
                    continue;
                };
                let entries = e
                    .iter()
                    .map(|(k, v)| (k.clone(), value_to_ini(v)))
                    .collect();
                doc.sections.push(IniSection {
                    name: name.clone(),
                    entries,
                });
            }
        }
        doc
    }

    /// Vers le JSON attendu par le client LoL, en reprenant le type des valeurs actuelles du client.
    pub fn to_json_like(&self, current: &Value) -> Value {
        let mut out = Map::new();
        for s in &self.sections {
            let mut sec = Map::new();
            for (k, v) in &s.entries {
                let existing = current.get(&s.name).and_then(|x| x.get(k));
                sec.insert(k.clone(), ini_to_value(v, existing));
            }
            out.insert(s.name.clone(), Value::Object(sec));
        }
        Value::Object(out)
    }
}

fn value_to_ini(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => {
            if *b {
                "1".into()
            } else {
                "0".into()
            }
        }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else {
                format!("{:.4}", n.as_f64().unwrap_or(0.0))
            }
        }
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn ini_to_value(s: &str, existing: Option<&Value>) -> Value {
    match existing {
        Some(Value::Bool(_)) => Value::Bool(matches!(s, "1" | "true" | "True")),
        Some(Value::Number(_)) => s
            .parse::<i64>()
            .map(Value::from)
            .or_else(|_| s.parse::<f64>().map(Value::from))
            .unwrap_or_else(|_| Value::String(s.to_string())),
        _ => Value::String(s.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_roundtrip_preserves_order() {
        let text = "[GameEvents]\nevtCastSpell1=[q]\nevtCastSpell2=[w]\n\n[HUDEvents]\nevtHoldShowScoreBoard=[Tab]\n";
        let doc = IniDoc::parse(text);
        assert_eq!(doc.sections.len(), 2);
        assert_eq!(
            doc.sections[0].entries[1],
            ("evtCastSpell2".into(), "[w]".into())
        );
        assert_eq!(IniDoc::parse(&doc.to_text()), doc);
    }

    #[test]
    fn merge_replaces_only_named_sections() {
        let mut base = IniDoc::parse("[A]\nx=1\nkeep=1\n[B]\ny=1\n");
        let other = IniDoc::parse("[A]\nx=2\n[B]\ny=2\n");
        base.merge_sections(&other, &["A"]);
        assert_eq!(
            base.section("A").unwrap().entries,
            vec![("x".into(), "2".into()), ("keep".into(), "1".into())]
        );
        assert_eq!(base.section("B").unwrap().entries[0].1, "1");
    }

    #[test]
    fn json_types_follow_client() {
        let doc = IniDoc::parse("[HUD]\nShowTimestamps=1\nGlobalScale=0.5000\nName=abc\n");
        let current = json!({"HUD": {"ShowTimestamps": false, "GlobalScale": 0.2}});
        let v = doc.to_json_like(&current);
        assert_eq!(v["HUD"]["ShowTimestamps"], json!(true));
        assert_eq!(v["HUD"]["GlobalScale"], json!(0.5));
        assert_eq!(v["HUD"]["Name"], json!("abc"));
        let back = IniDoc::from_json(&json!({"HUD": {"A": true, "B": 0.25, "C": 3}}));
        assert_eq!(
            back.sections[0].entries,
            vec![
                ("A".into(), "1".into()),
                ("B".into(), "0.2500".into()),
                ("C".into(), "3".into())
            ]
        );
    }
}
