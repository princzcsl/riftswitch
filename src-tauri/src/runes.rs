//! Pages de runes LoL : lecture / écriture via le client (LCU), bibliothèque dans le coffre,
//! codes de partage courts (`RSR1.` + ids séparés par des points, lisibles et sans secret).

use serde_json::{json, Value};

use crate::error::{AppError, AppResult};
use crate::riot::local_api::LocalApi;
use crate::util::now_ms;
use crate::vault::model::RunePage;

pub async fn client_pages(api: &LocalApi) -> AppResult<Vec<RunePage>> {
    let v = api.get("/lol-perks/v1/pages").await?;
    let pages = v.as_array().cloned().unwrap_or_default();
    Ok(pages
        .iter()
        .filter(|p| {
            p.get("isEditable")
                .and_then(|x| x.as_bool())
                .unwrap_or(true)
        })
        .map(|p| RunePage {
            id: p.get("id").map(|x| x.to_string()).unwrap_or_default(),
            name: p
                .get("name")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string(),
            primary_style_id: p
                .get("primaryStyleId")
                .and_then(|x| x.as_i64())
                .unwrap_or(0),
            sub_style_id: p.get("subStyleId").and_then(|x| x.as_i64()).unwrap_or(0),
            selected_perk_ids: p
                .get("selectedPerkIds")
                .and_then(|x| x.as_array())
                .map(|a| a.iter().filter_map(|i| i.as_i64()).collect())
                .unwrap_or_default(),
            champion: None,
            source_account: None,
            created_at: now_ms(),
        })
        .collect())
}

/// Ajoute une page dans le client. Si le nombre maximal est atteint, remplace la page éditable
/// la plus ancienne (celle en dernier dans la liste), comme le fait le client lui-même.
pub async fn push_page(api: &LocalApi, page: &RunePage) -> AppResult<()> {
    let body = json!({
        "name": page.name.chars().take(25).collect::<String>(),
        "primaryStyleId": page.primary_style_id,
        "subStyleId": page.sub_style_id,
        "selectedPerkIds": page.selected_perk_ids,
        "current": true,
    });
    if api.post("/lol-perks/v1/pages", &body).await.is_ok() {
        return Ok(());
    }
    let pages = api.get("/lol-perks/v1/pages").await?;
    let victim = pages
        .as_array()
        .and_then(|a| {
            a.iter().rev().find(|p| {
                p.get("isDeletable")
                    .and_then(|x| x.as_bool())
                    .unwrap_or(false)
            })
        })
        .and_then(|p| p.get("id").cloned())
        .ok_or_else(|| AppError::code("rune_pages_full"))?;
    api.delete(&format!("/lol-perks/v1/pages/{}", value_id(&victim)))
        .await?;
    api.post("/lol-perks/v1/pages", &body).await.map(|_| ())
}

fn value_id(v: &Value) -> String {
    v.as_i64()
        .map(|i| i.to_string())
        .unwrap_or_else(|| v.to_string())
}

pub fn to_code(p: &RunePage) -> String {
    let perks: Vec<String> = p.selected_perk_ids.iter().map(|i| i.to_string()).collect();
    let name: String = p
        .name
        .chars()
        .filter(|c| *c != '.' && *c != '|')
        .take(25)
        .collect();
    format!(
        "RSR1.{}.{}.{}|{}",
        p.primary_style_id,
        p.sub_style_id,
        perks.join("."),
        name
    )
}

pub fn from_code(code: &str) -> AppResult<RunePage> {
    let body = code
        .trim()
        .strip_prefix("RSR1.")
        .ok_or_else(|| AppError::code("code_invalid"))?;
    let (ids, name) = body.split_once('|').unwrap_or((body, "RiftSwitch"));
    let nums: Vec<i64> = ids
        .split('.')
        .map(|s| s.parse::<i64>())
        .collect::<Result<_, _>>()
        .map_err(|_| AppError::code("code_invalid"))?;
    if nums.len() < 3 || nums.len() > 12 {
        return Err(AppError::code("code_invalid"));
    }
    Ok(RunePage {
        id: uuid::Uuid::new_v4().to_string(),
        name: if name.trim().is_empty() {
            "RiftSwitch".into()
        } else {
            name.trim().to_string()
        },
        primary_style_id: nums[0],
        sub_style_id: nums[1],
        selected_perk_ids: nums[2..].to_vec(),
        created_at: now_ms(),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_roundtrip() {
        let p = RunePage {
            name: "Ahri mid".into(),
            primary_style_id: 8100,
            sub_style_id: 8300,
            selected_perk_ids: vec![8112, 8139, 8138, 8135, 8345, 8347, 5008, 5008, 5002],
            ..Default::default()
        };
        let code = to_code(&p);
        let back = from_code(&code).unwrap();
        assert_eq!(back.name, "Ahri mid");
        assert_eq!(back.selected_perk_ids, p.selected_perk_ids);
        assert!(from_code("RSR1.a.b|x").is_err());
    }
}
