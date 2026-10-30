//! Host presentation fixture checks; this is not a runtime renderer or patch API.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn fixture() -> Value {
    serde_json::from_str(include_str!("figure_contract.json")).unwrap()
}
fn evidence_contract() -> Value {
    serde_json::from_str(include_str!("semantic_contract.json")).unwrap()
}
fn ids<'a>(
    plan: &'a Value,
    section: &str,
    global: &mut BTreeSet<String>,
) -> Result<BTreeSet<&'a str>, String> {
    let mut found = BTreeSet::new();
    for item in plan[section].as_array().ok_or("missing section")? {
        let id = item["id"].as_str().ok_or("missing id")?;
        if id.is_empty()
            || !id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            || !global.insert(id.into())
        {
            return Err("invalid or duplicate id".into());
        }
        found.insert(id);
    }
    Ok(found)
}
fn references(value: &Value, known: &BTreeSet<&str>) -> Result<(), String> {
    for id in value.as_array().ok_or("missing references")? {
        if !known.contains(id.as_str().ok_or("invalid reference")?) {
            return Err("dangling reference".into());
        }
    }
    Ok(())
}
fn validate(plan: &Value) -> Result<(), String> {
    let contract = evidence_contract();
    let mut global = BTreeSet::new();
    ids(plan, "panels", &mut global)?;
    let claims = ids(plan, "claims", &mut global)?;
    let nodes = ids(plan, "nodes", &mut global)?;
    let edges = ids(plan, "edges", &mut global)?;
    let ledger = plan["evidence"].as_object().ok_or("missing evidence")?;
    let evidence: BTreeSet<_> = ledger.keys().map(String::as_str).collect();
    for witness in ledger.values() {
        let known = match witness["kind"].as_str() {
            Some("source_selection") => contract["selections"]
                .as_array()
                .unwrap()
                .iter()
                .any(|s| s["id"] == witness["ref"]),
            Some("import_fact") => contract["import_facts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| {
                    f["from"] == witness["from"]
                        && f["target"] == witness["target"]
                        && f["line"] == witness["line"]
                }),
            _ => false,
        };
        if !known {
            return Err("unknown evidence".into());
        }
    }
    let claim_map: BTreeMap<_, _> = plan["claims"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| (c["id"].as_str().unwrap(), c))
        .collect();
    for claim in claim_map.values() {
        references(&claim["evidence"], &evidence)?;
        match claim["status"].as_str() {
            Some("unknown") => {}
            Some("observed" | "gpt_interpretation")
                if !claim["evidence"].as_array().unwrap().is_empty() => {}
            _ => return Err("claim needs status and evidence".into()),
        }
    }
    let node_map: BTreeMap<_, _> = plan["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["id"].as_str().unwrap(), n))
        .collect();
    for node in node_map.values() {
        if node
            .as_object()
            .unwrap()
            .keys()
            .any(|k| !matches!(k.as_str(), "id" | "component" | "label" | "claim_ids"))
        {
            return Err("node must be declarative, without geometry/code".into());
        }
        references(&node["claim_ids"], &claims)?;
        for id in node["claim_ids"].as_array().unwrap() {
            let claim = claim_map[id.as_str().unwrap()];
            if claim["status"] == "unknown" {
                continue;
            }
            let source_match = claim["evidence"].as_array().unwrap().iter().any(|id| {
                let witness = &ledger[id.as_str().unwrap()];
                witness["kind"] == "source_selection"
                    && contract["selections"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|s| s["id"] == witness["ref"] && s["path"] == node["component"])
            });
            if !source_match {
                return Err("role citation does not support this component".into());
            }
        }
    }
    for edge in plan["edges"].as_array().unwrap() {
        let from = edge["from"].as_str().ok_or("missing endpoint")?;
        let to = edge["to"].as_str().ok_or("missing endpoint")?;
        if !nodes.contains(from) || !nodes.contains(to) {
            return Err("dangling endpoint".into());
        }
        references(&edge["claim_ids"], &claims)?;
        match edge["basis"].as_str() {
            Some("layout_candidate" | "literal_relative_path") => {
                let observation = edge["observation"].as_str().ok_or("missing observation")?;
                let witness = ledger.get(observation).ok_or("unknown observation")?;
                let matching = contract["import_facts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|f| {
                        f["from"] == witness["from"]
                            && f["target"] == witness["target"]
                            && f["line"] == witness["line"]
                            && f["from"] == node_map[from]["component"]
                            && f["resolved"] == node_map[to]["component"]
                            && f["resolution"] == edge["basis"]
                    });
                if !matching {
                    return Err("edge observation mismatch".into());
                }
            }
            Some("gpt_interpretation") if edge.get("observation").is_none() => {
                if edge["claim_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|id| claim_map[id.as_str().unwrap()]["status"] == "observed")
                {
                    return Err("semantic overlay promoted to observed fact".into());
                }
            }
            _ => return Err("unsupported or promoted edge basis".into()),
        }
    }
    for panel in plan["panels"].as_array().unwrap() {
        references(&panel["claim_ids"], &claims)?;
        references(&panel["node_ids"], &nodes)?;
        references(&panel["edge_ids"], &edges)?;
    }
    if plan["layout"] != json!({"strategy":"native-auto","direction":"LR"}) {
        return Err("fixture delegates layout, not coordinates".into());
    }
    Ok(())
}

#[test]
fn declarative_figure_links_prose_roles_edges_and_real_evidence() {
    assert_eq!(validate(&fixture()), Ok(()));
    let mut unknown = fixture();
    unknown["claims"][1]["status"] = json!("unknown");
    unknown["claims"][1]["evidence"] = json!([]);
    unknown["claims"][1]["text"] = json!("選択sourceが未確認");
    unknown["nodes"][1]["label"] = json!("責務未確認");
    assert_eq!(validate(&unknown), Ok(()));
}

#[test]
fn invalid_ids_citations_endpoints_and_geometry_are_rejected_in_fixture_contract() {
    let mut invalid = fixture();
    invalid["panels"][2]["id"] = json!("responsibilities");
    assert!(validate(&invalid).is_err());
    let mut invalid = fixture();
    invalid["claims"][0]["evidence"] = json!([]);
    assert!(validate(&invalid).is_err());
    let mut invalid = fixture();
    invalid["claims"][0]["evidence"] = json!(["missing"]);
    assert!(validate(&invalid).is_err());
    let mut invalid = fixture();
    invalid["edges"][0]["to"] = json!("missing");
    assert!(validate(&invalid).is_err());
    let mut invalid = fixture();
    invalid["edges"][0]["observation"] = json!("F2");
    assert!(validate(&invalid).is_err());
    let mut invalid = fixture();
    invalid["claims"][1]["evidence"] = json!(["S2"]);
    assert!(validate(&invalid).is_err());
    let mut invalid = fixture();
    invalid["claims"][5]["status"] = json!("observed");
    assert!(validate(&invalid).is_err());
    let mut invalid = fixture();
    invalid["nodes"][0]["x"] = json!(120);
    assert!(validate(&invalid).is_err());
}

#[test]
fn focused_revision_keeps_identity_and_unrelated_evidence_after_title_or_order_changes() {
    let before = fixture();
    let mut after = before.clone();
    let panels = after["panels"].as_array_mut().unwrap();
    panels.insert(
        0,
        json!({"id":"scope","title":"確認範囲","claim_ids":[],"node_ids":[],"edge_ids":[]}),
    );
    let target = panels.iter_mut().find(|p| p["id"] == "impact").unwrap();
    target["title"] = json!("受付・失敗条件の候補影響");
    let impact = after["claims"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["id"] == "impact-validator")
        .unwrap();
    impact["text"] = json!("受付・失敗条件への影響は候補であり、実行時の結果は未確認");
    for id in ["responsibilities", "dependencies"] {
        assert_eq!(
            before["panels"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["id"] == id),
            after["panels"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["id"] == id)
        );
    }
    for claim in before["claims"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["id"] != "impact-validator")
    {
        assert_eq!(
            Some(claim),
            after["claims"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["id"] == claim["id"])
        );
    }
    for key in ["nodes", "edges", "evidence"] {
        assert_eq!(before[key], after[key]);
    }
    let old_impact = before["claims"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "impact-validator")
        .unwrap();
    let new_impact = after["claims"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "impact-validator")
        .unwrap();
    assert_eq!(old_impact["evidence"], new_impact["evidence"]);
    assert_ne!(old_impact["text"], new_impact["text"]);
    assert_eq!(validate(&after), Ok(()));
}
