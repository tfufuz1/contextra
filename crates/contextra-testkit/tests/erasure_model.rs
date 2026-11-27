//! Modelltest (13 §6): ein unabhängiges, persistentes Mini-System über `CrashVfs`
//! muss nach beliebigen Operationsfolgen mit Löschungen und Neustarts dem Modell gleichen.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_ports::{Vfs, VfsOpenOptions};
use contextra_testkit::canary::Canary;
use contextra_testkit::crash_vfs::*;
use contextra_testkit::erasure_model::*;
use contextra_types::{DocId, EraseTarget, TenantId};
use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const LOG: &str = "/db/log";

/// `.into()` ist nur ohne Feature `docid-128` eine Identität.
#[allow(clippy::useless_conversion)]
fn did(n: u64) -> DocId {
    DocId(n.into())
}

/// Eine Logzeile als Tab-Felder; Testeingaben enthalten weder Tab noch Zeilenumbruch.
type Line = Vec<String>;

struct Sut {
    vfs: CrashVfs,
    /// Defekt: `erase` lässt alte Versionen stehen.
    keep_history: bool,
}

fn tenant(s: &str) -> TenantId {
    TenantId::try_new(s.parse().unwrap()).unwrap()
}

fn id(l: &Line) -> (TenantId, String, DocId) {
    (tenant(&l[2]), l[3].clone(), DocId(l[4].parse().unwrap()))
}

fn terms(s: &str) -> BTreeSet<String> {
    s.split(',')
        .filter(|t| !t.is_empty())
        .map(String::from)
        .collect()
}

impl Sut {
    fn new() -> Self {
        let vfs = CrashVfs::new(CrashSeed(11));
        vfs.create_dir_all(Path::new("/db")).unwrap();
        vfs.sync_dir(Path::new("/")).unwrap();
        vfs.write_atomic(Path::new(LOG), b"").unwrap();
        Self {
            vfs,
            keep_history: false,
        }
    }

    fn lines(&self) -> Result<Vec<Line>, String> {
        let raw = self.vfs.read(Path::new(LOG)).map_err(|e| e.to_string())?;
        let text = String::from_utf8(raw).map_err(|e| e.to_string())?;
        // Nur vollständige Zeilen zählen (Torn Tail).
        let end = text.rfind('\n').map_or(0, |i| i + 1);
        Ok(text[..end]
            .lines()
            .map(|l| l.split('\t').map(String::from).collect())
            .collect())
    }

    fn append(&self, line: Line) -> Result<(), String> {
        let path = Path::new(LOG);
        let opts = VfsOpenOptions {
            write: true,
            append: true,
            ..Default::default()
        };
        let f = self.vfs.open(path, &opts).map_err(|e| e.to_string())?;
        let data = format!("{}\n", line.join("\t"));
        f.write_at(data.as_bytes(), 0).map_err(|e| e.to_string())?;
        f.sync_data().map_err(|e| e.to_string())
    }

    fn live(lines: &[Line]) -> BTreeMap<(TenantId, String, DocId), Line> {
        let mut live = BTreeMap::new();
        for l in lines {
            match l[0].as_str() {
                "W" => {
                    live.insert(id(l), l.clone());
                }
                "D" => {
                    live.remove(&id(l));
                }
                _ => {}
            }
        }
        live
    }

    fn next_seq(lines: &[Line]) -> u64 {
        // "S" hält den Zähler auch dann, wenn die letzten Versionen gelöscht wurden.
        lines
            .iter()
            .filter(|l| matches!(l[0].as_str(), "W" | "D" | "S"))
            .map(|l| l[1].parse::<u64>().unwrap())
            .max()
            .unwrap_or(0)
            + 1
    }

    fn apply(&mut self, op: &Op) -> Result<(), String> {
        let lines = self.lines()?;
        let live = Self::live(&lines);
        let seq = Self::next_seq(&lines).to_string();
        let head = |kind: &str, t: &TenantId, c: &str, d: DocId| -> Line {
            vec![
                kind.into(),
                seq.clone(),
                t.as_u64().to_string(),
                c.into(),
                d.0.to_string(),
            ]
        };
        match op {
            Op::Insert {
                tenant,
                collection,
                doc,
                record,
            }
            | Op::Update {
                tenant,
                collection,
                doc,
                record,
            } => {
                let key = (*tenant, collection.clone(), *doc);
                if matches!(op, Op::Update { .. }) && !live.contains_key(&key) {
                    return Ok(());
                }
                let mut l = head("W", tenant, collection, *doc);
                l.push(record.key.clone());
                l.push(String::from_utf8(record.body.clone()).unwrap());
                l.push(String::from_utf8(record.embedding.clone()).unwrap());
                l.push(record.terms.iter().cloned().collect::<Vec<_>>().join(","));
                self.append(l)
            }
            Op::Delete {
                tenant,
                collection,
                doc,
            } => {
                if !live.contains_key(&(*tenant, collection.clone(), *doc)) {
                    return Ok(());
                }
                self.append(head("D", tenant, collection, *doc))
            }
            Op::Link {
                obj,
                collection,
                doc,
            } => {
                if !live.contains_key(&(obj.tenant, collection.clone(), *doc)) {
                    return Ok(());
                }
                let mut l = head("L", &obj.tenant, collection, *doc);
                l.push(obj.name.clone());
                self.append(l)
            }
            Op::Checkpoint { name } => {
                let colls: BTreeSet<String> = live
                    .keys()
                    .map(|(t, c, _)| format!("{}:{c}", t.as_u64()))
                    .collect();
                let at = (Self::next_seq(&lines) - 1).to_string();
                self.append(vec![
                    "C".into(),
                    at,
                    name.clone(),
                    colls.into_iter().collect::<Vec<_>>().join(","),
                ])
            }
        }
    }

    /// Schreibt das Log ohne das Ziel atomar neu.
    fn erase(&mut self, target: &EraseTarget) -> Result<(), String> {
        let lines = self.lines()?;
        let hit: BTreeSet<_> = lines
            .iter()
            .filter(|l| l[0] == "W")
            .filter(|l| {
                let (t, c, _) = id(l);
                match target {
                    EraseTarget::Document {
                        tenant,
                        collection,
                        key,
                    } => t == *tenant && c == *collection && l[5] == *key,
                    EraseTarget::Collection { tenant, collection } => {
                        t == *tenant && c == *collection
                    }
                    EraseTarget::Tenant { tenant } => t == *tenant,
                    _ => false,
                }
            })
            .map(id)
            .collect();
        let tainted: BTreeSet<String> = lines
            .iter()
            .filter(|l| l[0] == "C")
            .filter(|c| {
                let at: u64 = c[1].parse().unwrap();
                let before: Vec<Line> = lines
                    .iter()
                    .filter(|l| matches!(l[0].as_str(), "W" | "D"))
                    .filter(|l| l[1].parse::<u64>().unwrap() <= at)
                    .cloned()
                    .collect();
                Self::live(&before).keys().any(|k| hit.contains(k))
            })
            .map(|c| c[2].clone())
            .collect();
        let high = Self::next_seq(&lines) - 1;
        let mut out = format!("S\t{high}\n");
        for l in &lines {
            let drop = match l[0].as_str() {
                "W" if self.keep_history => false,
                "W" | "D" | "L" => hit.contains(&id(l)),
                "C" => tainted.contains(&l[2]),
                _ => true,
            };
            if !drop {
                out.push_str(&l.join("\t"));
                out.push('\n');
            }
        }
        self.vfs
            .write_atomic(Path::new(LOG), out.as_bytes())
            .map_err(|e| e.to_string())
    }

    fn restart(&mut self) {
        self.vfs = self.vfs.restart(CrashPolicy::DropUnsynced);
    }
}

impl SystemUnderTest for Sut {
    fn inventory(&self) -> Result<Inventory, String> {
        let lines = self.lines()?;
        let mut out = Inventory::new();
        // Letzter Checkpoint je Name gewinnt.
        let mut checkpoints = BTreeMap::new();
        for l in &lines {
            match l[0].as_str() {
                "W" | "D" => {
                    let (t, c, d) = id(l);
                    out.insert(Entry::Version {
                        key: kv_key(t, &c, d),
                        seq: l[1].parse().unwrap(),
                        value: (l[0] == "W").then(|| l[6].clone().into_bytes()),
                    });
                }
                "L" => {
                    let (t, c, d) = id(l);
                    out.insert(Entry::Edge {
                        obj: ObjRef {
                            tenant: t,
                            name: l[5].clone(),
                        },
                        collection: c,
                        doc: d,
                    });
                }
                "C" => {
                    checkpoints.insert(l[2].clone(), (l[1].clone(), l[3].clone()));
                }
                _ => {}
            }
        }
        for ((t, c, d), l) in Self::live(&lines) {
            let record = DocRecord {
                key: l[5].clone(),
                body: l[6].clone().into_bytes(),
                embedding: l[7].clone().into_bytes(),
                terms: terms(&l[8]),
            };
            for term in &record.terms {
                out.insert(Entry::Posting {
                    tenant: t,
                    collection: c.clone(),
                    term: term.clone(),
                    doc: d,
                });
            }
            out.insert(Entry::Doc {
                tenant: t,
                collection: c,
                doc: d,
                record,
            });
        }
        for (name, (seq, colls)) in checkpoints {
            out.insert(Entry::Checkpoint {
                name,
                seq: seq.parse().unwrap(),
                collections: terms(&colls)
                    .iter()
                    .map(|s| {
                        let (t, c) = s.split_once(':').unwrap();
                        (tenant(t), c.to_string())
                    })
                    .collect(),
            });
        }
        Ok(out)
    }

    fn visible(&self, t: TenantId) -> Result<BTreeSet<(String, DocId)>, String> {
        Ok(Self::live(&self.lines()?)
            .into_keys()
            .filter(|k| k.0 == t)
            .map(|(_, c, d)| (c, d))
            .collect())
    }
}

#[derive(Debug, Clone)]
enum Step {
    Op(Op),
    Erase(EraseTarget),
    Restart,
}

fn tenants() -> impl Strategy<Value = TenantId> {
    (1u64..=3).prop_map(|t| TenantId::try_new(t).unwrap())
}

fn collections() -> impl Strategy<Value = String> {
    prop::sample::select(vec!["a", "b", "c"]).prop_map(String::from)
}

fn user_key(doc: u64) -> String {
    format!("key{doc}")
}

fn record(doc: u64, v: u8) -> DocRecord {
    DocRecord {
        key: user_key(doc),
        body: format!("body{doc}-{v}").into_bytes(),
        embedding: format!("emb{v}").into_bytes(),
        terms: [format!("t{}", v % 3), "common".to_string()].into(),
    }
}

fn steps() -> impl Strategy<Value = Step> {
    let at = || (tenants(), collections(), 0u64..4);
    prop_oneof![
        6 => (at(), any::<u8>(), any::<bool>()).prop_map(|((tenant, collection, d), v, upd)| {
            let (doc, record) = (did(d), record(d, v));
            Step::Op(if upd {
                Op::Update { tenant, collection, doc, record }
            } else {
                Op::Insert { tenant, collection, doc, record }
            })
        }),
        2 => at().prop_map(|(tenant, collection, d)| {
            Step::Op(Op::Delete { tenant, collection, doc: did(d) })
        }),
        2 => (at(), 0u8..3).prop_map(|((tenant, collection, d), n)| {
            let obj = ObjRef { tenant, name: format!("obj{n}") };
            Step::Op(Op::Link { obj, collection, doc: did(d) })
        }),
        1 => (0u8..3).prop_map(|n| Step::Op(Op::Checkpoint { name: format!("cp{n}") })),
        2 => at().prop_map(|(tenant, collection, d)| {
            Step::Erase(EraseTarget::Document { tenant, collection, key: user_key(d) })
        }),
        1 => (tenants(), collections()).prop_map(|(tenant, collection)| {
            Step::Erase(EraseTarget::Collection { tenant, collection })
        }),
        1 => tenants().prop_map(|tenant| Step::Erase(EraseTarget::Tenant { tenant })),
        1 => Just(Step::Restart),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn system_matches_model(script in prop::collection::vec(steps(), 0..=60)) {
        let mut model = ErasureModel::new();
        let mut sut = Sut::new();
        for step in &script {
            match step {
                Step::Op(op) => {
                    model.apply(op.clone());
                    sut.apply(op).map_err(TestCaseError::fail)?;
                }
                Step::Erase(target) => {
                    model.erase(target);
                    sut.erase(target).map_err(TestCaseError::fail)?;
                    // Erase ∘ Erase = Erase.
                    let once = (model.inventory(), sut.inventory().map_err(TestCaseError::fail)?);
                    model.erase(target);
                    sut.erase(target).map_err(TestCaseError::fail)?;
                    prop_assert_eq!(&once.0, &model.inventory());
                    prop_assert_eq!(once.1, sut.inventory().map_err(TestCaseError::fail)?);
                }
                Step::Restart => sut.restart(),
            }
        }
        sut.restart();
        model.assert_matches(&sut).map_err(|d| TestCaseError::fail(d.to_string()))?;
    }
}

fn insert(model: &mut ErasureModel, sut: &mut Sut, t: u64, doc: u64, body: &[u8]) {
    let mut rec = record(doc, 1);
    rec.body = body.to_vec();
    let op = Op::Insert {
        tenant: TenantId::try_new(t).unwrap(),
        collection: "a".into(),
        doc: did(doc),
        record: rec,
    };
    model.apply(op.clone());
    sut.apply(&op).unwrap();
}

#[test]
fn retire_is_not_erase_and_canary_disappears() {
    let canary = Canary::from_seed(5, 1);
    let (mut model, mut sut) = (ErasureModel::new(), Sut::new());
    let t = TenantId::try_new(1).unwrap();
    insert(&mut model, &mut sut, 1, 1, &canary.value_needle);
    insert(&mut model, &mut sut, 2, 1, b"neighbor");
    let seq = 1;
    let delete = Op::Delete {
        tenant: t,
        collection: "a".into(),
        doc: did(1),
    };
    model.apply(delete.clone());
    sut.apply(&delete).unwrap();
    assert!(model.get_at_seq(t, "a", did(1), seq).is_some());
    assert!(model.expected_visible(t).is_empty());
    // Negativkontrolle: vor dem Erase ist die Nadel auffindbar.
    assert!(!sut
        .vfs
        .snapshot_tree()
        .find(&canary.value_needle)
        .is_empty());

    let target = EraseTarget::Document {
        tenant: t,
        collection: "a".into(),
        key: user_key(1),
    };
    model.erase(&target);
    sut.erase(&target).unwrap();
    for s in 0..=model.seq() {
        assert_eq!(model.get_at_seq(t, "a", did(1), s), None);
    }
    assert!(sut
        .vfs
        .snapshot_tree()
        .find(&canary.value_needle)
        .is_empty());
    model.assert_matches(&sut).unwrap();
}

#[test]
fn model_detects_erase_that_keeps_history() {
    let (mut model, mut sut) = (ErasureModel::new(), Sut::new());
    sut.keep_history = true;
    insert(&mut model, &mut sut, 1, 1, b"secret");
    let target = EraseTarget::Tenant {
        tenant: TenantId::try_new(1).unwrap(),
    };
    model.erase(&target);
    sut.erase(&target).unwrap();
    let diff = model.assert_matches(&sut).unwrap_err();
    assert!(!diff.unexpected.is_empty() && diff.missing.is_empty());
}

#[test]
fn erase_is_atomic_at_every_crash_point() {
    let target = EraseTarget::Tenant {
        tenant: TenantId::try_new(1).unwrap(),
    };
    let build = |vfs: &CrashVfs| {
        let mut model = ErasureModel::new();
        let mut sut = Sut {
            vfs: vfs.clone(),
            keep_history: false,
        };
        let mut setup = || -> Result<(), String> {
            sut.vfs
                .create_dir_all(Path::new("/db"))
                .and_then(|()| sut.vfs.sync_dir(Path::new("/")))
                .and_then(|()| sut.vfs.write_atomic(Path::new(LOG), b""))
                .map_err(|e| e.to_string())?;
            for t in [1, 2] {
                let op = Op::Insert {
                    tenant: TenantId::try_new(t).unwrap(),
                    collection: "a".into(),
                    doc: did(t),
                    record: record(t, 1),
                };
                model.apply(op.clone());
                sut.apply(&op)?;
            }
            Ok(())
        };
        let result = setup();
        (model, sut, result)
    };
    let reference = CrashVfs::new(CrashSeed(1));
    let (before, _, ok) = build(&reference);
    ok.unwrap();
    let setup_ops = reference.op_count();
    let mut after = before.clone();
    after.erase(&target);

    let report = for_each_crash_point_with(
        &CrashLoopConfig {
            seed: CrashSeed(1),
            ..Default::default()
        },
        |v| {
            let v = v.clone();
            let target = target.clone();
            Scenario::new(move || {
                let (_, mut sut, ok) = build(&v);
                ok?;
                sut.erase(&target)
            })
        },
        &[
            CrashPolicy::DropUnsynced,
            CrashPolicy::TornTail { sector: 1 },
            CrashPolicy::PartialNamespace,
            CrashPolicy::KeepAll,
        ],
        |restarted, point| {
            if point.n < setup_ops {
                return Ok(());
            }
            let mut sut = Sut {
                vfs: restarted.clone(),
                keep_history: false,
            };
            // Vor der Wiederaufnahme: ganz alt oder ganz neu.
            if before.assert_matches(&sut).is_err() {
                after.assert_matches(&sut).map_err(|d| d.to_string())?;
            }
            sut.erase(&target)?;
            after.assert_matches(&sut).map_err(|d| d.to_string())
        },
    )
    .unwrap();
    assert!(report.total_ops > setup_ops);
}
