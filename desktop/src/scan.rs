//! The in-game probe scanner's clipboard, read the way the website reads a paste. A port of
//! `parseScan` in `frontend/src/lib/map/signatures.ts`, tested against the same cases;
//! change both together. The localized names are the website's own file.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::api::SignatureCatalog;

/// Localized category and site names mapped to their English catalog names. Sites are
/// keyed by English category, since one localized name can mean different sites in
/// different categories.
#[derive(Deserialize)]
struct ScanNames {
    categories: HashMap<String, String>,
    types: HashMap<String, HashMap<String, String>>,
}

fn names() -> &'static ScanNames {
    static NAMES: OnceLock<ScanNames> = OnceLock::new();
    NAMES.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../frontend/src/lib/signatures/translations.json"
        ))
        .expect("translations.json is valid")
    })
}

/// One row of a paste, as the server's paste command takes it. `group` is `None` when the
/// scanner had not classified the signature yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PastedSignature {
    pub signature_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_type_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// The signature group a catalog category stands for.
pub fn group_for_category(category_id: i64) -> &'static str {
    match category_id {
        1 => "wormhole",
        2 => "data",
        3 => "relic",
        4 => "combat",
        5 => "gas",
        6 => "ore",
        7 => "homefront",
        8 => "faction_warfare",
        _ => "unknown",
    }
}

/// Tab-separated rows of `id, scan group, category, type, signal, distance`. Rows under four
/// columns or without a seven-character id are skipped. A wormhole's type never comes from
/// a paste: the scanner only says "Unstable Wormhole", so the pilot picks it.
pub fn parse(text: &str, catalog: &SignatureCatalog) -> Vec<PastedSignature> {
    let names = names();
    let mut out = Vec::new();
    for line in text.split('\n') {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 4 {
            continue;
        }
        let sid = cols[0].trim();
        if sid.chars().count() != 7 {
            continue;
        }
        let category = match_category(catalog, names, &normalize(cols[2]));
        let type_name = normalize(cols[3]);
        let mut type_id = None;
        if let Some(category) = category
            && group_for_category(category) != "wormhole"
            && !type_name.is_empty()
        {
            let category_name = catalog
                .categories
                .iter()
                .find(|c| c.id == category)
                .map(|c| c.name.as_str())
                .unwrap_or("");
            let english = names
                .types
                .get(category_name)
                .and_then(|t| t.get(&type_name))
                .unwrap_or(&type_name);
            type_id = catalog
                .types
                .iter()
                .find(|t| t.signature_category_id == category && &t.name == english)
                .map(|t| t.id);
        }
        out.push(PastedSignature {
            signature_id: sid.to_string(),
            group: category.map(group_for_category),
            signature_type_id: type_id,
            name: (category.is_some() && type_id.is_none() && !type_name.is_empty())
                .then_some(type_name),
        });
    }
    out
}

/// Some client languages separate words with non-breaking spaces.
fn normalize(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn find_category(catalog: &SignatureCatalog, names: &ScanNames, name: &str) -> Option<i64> {
    let english = names.categories.get(name).map_or(name, String::as_str);
    catalog
        .categories
        .iter()
        .find(|c| c.name == english)
        .map(|c| c.id)
}

/// By the whole cell, else by its ` - ` segments, the first known one winning.
fn match_category(catalog: &SignatureCatalog, names: &ScanNames, name: &str) -> Option<i64> {
    if name.is_empty() {
        return None;
    }
    find_category(catalog, names, name).or_else(|| {
        name.split(" - ")
            .find_map(|segment| find_category(catalog, names, segment.trim()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{SignatureCategory, SignatureType};

    fn category(id: i64, name: &str) -> SignatureCategory {
        SignatureCategory {
            id,
            name: name.into(),
        }
    }

    fn kind(id: i64, name: &str, category: i64) -> SignatureType {
        SignatureType {
            id,
            signature: None,
            name: name.into(),
            signature_category_id: category,
            target_class: None,
            max_jump_mass: None,
            extra: None,
            spawn_areas: Vec::new(),
        }
    }

    fn catalog() -> SignatureCatalog {
        SignatureCatalog {
            categories: vec![
                category(1, "Wormhole"),
                category(2, "Data Site"),
                category(3, "Relic Site"),
                category(5, "Gas Site"),
            ],
            types: vec![
                SignatureType {
                    signature: Some("K162".into()),
                    ..kind(10, "Unstable Wormhole", 1)
                },
                kind(20, "Unsecured Frontier Receiver", 2),
                kind(30, "Forgotten Perimeter Coronation Site", 3),
            ],
        }
    }

    fn localized() -> SignatureCatalog {
        SignatureCatalog {
            categories: vec![
                category(1, "Wormhole"),
                category(2, "Data Site"),
                category(3, "Relic Site"),
                category(4, "Combat Site"),
                category(5, "Gas Site"),
                category(6, "Ore Site"),
                category(8, "Factional Warfare Site"),
            ],
            types: vec![
                kind(124, "Unsecured Perimeter Amplifier", 2),
                kind(65, "Ruined Sansha Temple Site", 3),
                kind(17, "Core Garrison", 4),
                kind(35, "Unstable Wormhole", 4),
                kind(136, "Barren Perimeter Reservoir", 5),
            ],
        }
    }

    fn row(
        id: &str,
        group: Option<&'static str>,
        ty: Option<i64>,
        name: Option<&str>,
    ) -> PastedSignature {
        PastedSignature {
            signature_id: id.into(),
            group,
            signature_type_id: ty,
            name: name.map(Into::into),
        }
    }

    #[test]
    fn parses_a_multi_line_paste() {
        let paste = [
            "ABC-123\tCosmic Signature\tWormhole\tUnstable Wormhole\t100.0%\t2.5 AU",
            "DEF-456\tCosmic Signature\tData Site\tUnsecured Frontier Receiver\t97.2%\t11.1 AU",
            "GHI-789\tCosmic Signature\tRelic Site\tForgotten Perimeter Coronation Site\t54.8%\t3.2 AU",
        ]
        .join("\n");
        assert_eq!(
            parse(&paste, &catalog()),
            vec![
                row("ABC-123", Some("wormhole"), None, Some("Unstable Wormhole")),
                row("DEF-456", Some("data"), Some(20), None),
                row("GHI-789", Some("relic"), Some(30), None),
            ]
        );
    }

    #[test]
    fn keeps_partially_scanned_rows() {
        let paste = "JKL-012\tCosmic Signature\t\t\t2.9%\t8.7 AU\nMNO-345\tCosmic Signature\tGas Site\t\t12.0%\t1.4 AU";
        assert_eq!(
            parse(paste, &catalog()),
            vec![
                row("JKL-012", None, None, None),
                row("MNO-345", Some("gas"), None, None),
            ]
        );
    }

    #[test]
    fn keeps_an_unmatched_type_name_as_the_raw_name() {
        let paste = "PQR-678\tCosmic Signature\tData Site\tSome New Site\t80%\t1 AU";
        assert_eq!(
            parse(paste, &catalog()),
            vec![row("PQR-678", Some("data"), None, Some("Some New Site"))]
        );
    }

    #[test]
    fn matches_a_category_by_its_segments() {
        let paste = "STU-901\tCosmic Signature\tGuristas - Data Site\t\t9%\t3 AU";
        assert_eq!(
            parse(paste, &catalog()),
            vec![row("STU-901", Some("data"), None, None)]
        );
    }

    #[test]
    fn skips_malformed_lines() {
        let paste = [
            "",
            "not a scanner line at all",
            "AB-12\tCosmic Signature\tWormhole\tUnstable Wormhole\t100%\t1 AU",
            "ABC-123\tCosmic Signature",
            "VWX-234\tCosmic Signature\tRelic Site\tForgotten Perimeter Coronation Site\t54.8%\t3.2 AU",
        ]
        .join("\n");
        assert_eq!(
            parse(&paste, &catalog()),
            vec![row("VWX-234", Some("relic"), Some(30), None)]
        );
    }

    #[test]
    fn files_faction_warfare_anomalies_with_the_pasted_name() {
        let mut fw = catalog();
        fw.categories.push(category(6, "Ore Site"));
        fw.categories.push(category(8, "Factional Warfare Site"));
        let paste = [
            "BBL-893\tCosmic Anomaly\tFactional Warfare Site - Combat Site\tAmarr Scout BSC-1\t100,0%\t6,84 AU",
            "CBA-620\tCosmic Anomaly\tOre Site\tGlacial Mass Belt\t100,0%\t7,02 AU",
            "DXA-556\tCosmic Signature\t\t\t0,0%\t6,51 AU",
            "MSA-264\tCosmic Anomaly\tFactional Warfare Site - Combat Site\tAmarr Moderate NVY-3\t100,0%\t60,60 AU",
        ]
        .join("\n");
        assert_eq!(
            parse(&paste, &fw),
            vec![
                row(
                    "BBL-893",
                    Some("faction_warfare"),
                    None,
                    Some("Amarr Scout BSC-1")
                ),
                row("CBA-620", Some("ore"), None, Some("Glacial Mass Belt")),
                row("DXA-556", None, None, None),
                row(
                    "MSA-264",
                    Some("faction_warfare"),
                    None,
                    Some("Amarr Moderate NVY-3")
                ),
            ]
        );
    }

    #[test]
    fn parses_nothing_from_empty_input() {
        assert!(parse("", &catalog()).is_empty());
    }

    #[test]
    fn parses_a_german_paste() {
        let paste = [
            "ABC-123\tKosmische Signatur\tDatengebiet\tUngesicherter Verstärker im Außenbereich\t100,0%\t4,2 AE",
            "DEF-456\tKosmische Signatur\tWurmloch\tInstabiles Wurmloch\t100,0%\t12,34 AE",
            "GHI-789\tKosmische Signatur\t\t\t0,0%\t6,51 AE",
        ]
        .join("\n");
        assert_eq!(
            parse(&paste, &localized()),
            vec![
                row("ABC-123", Some("data"), Some(124), None),
                row(
                    "DEF-456",
                    Some("wormhole"),
                    None,
                    Some("Instabiles Wurmloch")
                ),
                row("GHI-789", None, None, None),
            ]
        );
    }

    #[test]
    fn parses_russian_and_chinese_pastes() {
        let paste = [
            "ABC-123\tСкрытый сигнал\tАрхеологический район\tРазрушенный храмовый комплекс Sansha\t100,0%\t3,10 а.е.",
            "DEF-456\t空间信号\t战斗地点\t核心兵站\t100.0%\t5.03 AU",
        ]
        .join("\n");
        assert_eq!(
            parse(&paste, &localized()),
            vec![
                row("ABC-123", Some("relic"), Some(65), None),
                row("DEF-456", Some("combat"), Some(17), None),
            ]
        );
    }

    #[test]
    fn parses_a_clipboard_mixing_languages() {
        let paste = [
            "AAA-111\tCosmic Signature\tGas Site\tBarren Perimeter Reservoir\t100.0%\t5.03 AU",
            "BBB-222\tKosmische Signatur\tGasgebiet\tKarges Gasvorkommen\t100,0%\t4,2 AE",
            "CCC-333\t코즈믹 시그니처\t가스 사이트\t황량한 변방 가스 매장지\t100.0%\t1.20 AU",
            "DDD-444\t宇宙のシグネチャ\tガスサイト\t荒地の防御ライン貯水池\t100.0%\t2.40 AU",
            "EEE-555\tSignature cosmique\tSite de collecte de gaz\tRéservoir de périmètre aride\t100,0%\t7,00 UA",
            "FFF-666\tSeñal cósmica\tZona de gas\tReserva perimetral baldía\t100,0%\t8,00 UA",
        ]
        .join("\n");
        let rows = parse(&paste, &localized());
        assert_eq!(rows.len(), 6);
        for r in rows {
            assert_eq!(
                (r.group, r.signature_type_id, r.name),
                (Some("gas"), Some(136), None)
            );
        }
    }

    #[test]
    fn treats_non_breaking_spaces_like_spaces() {
        let paste = "ABC-123\tКосмическая аномалия\tБоевой\u{a0}район\tНеустойчивая\u{a0}червоточина\t100,0%\t1,00 а.е.";
        assert_eq!(
            parse(paste, &localized()),
            vec![row("ABC-123", Some("combat"), Some(35), None)]
        );
    }
}
