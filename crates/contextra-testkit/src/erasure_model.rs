//! # Referenzmodell der Löschkette (`erasure_model`)
//!
//! [`ErasureModel`] führt den Sollzustand aller Schichten (KV mit Historie, Dokumente,
//! Postings, Graphkanten, Checkpoints). Ein System unter Test legt sein vollständiges
//! [`Inventory`] offen; [`ErasureModel::assert_matches`] vergleicht auf Mengengleichheit.
//! Damit sind „nichts aus dem Scope in irgendeiner Schicht“ (kein Überschuss) und
//! „Nachbarn identisch“ (nichts fehlt) in einer Prüfung enthalten.

use crate::reference_model::ReferenceModel;
use contextra_types::{DocId, EraseTarget, TenantId};
use std::collections::{BTreeMap, BTreeSet};

/// Inhalt eines Dokuments.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocRecord {
    /// Nutzerschlüssel.
    pub key: String,
    /// Körper.
    pub body: Vec<u8>,
    /// Embedding-Rohbytes.
    pub embedding: Vec<u8>,
    /// Indexierte Terme.
    pub terms: BTreeSet<String>,
}

/// Graphobjekt, mandantengebunden.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjRef {
    /// Mandant.
    pub tenant: TenantId,
    /// Objektname.
    pub name: String,
}

/// Modelloperation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    /// Anlegen oder vollständig ersetzen.
    Insert {
        /// Mandant.
        tenant: TenantId,
        /// Collection.
        collection: String,
        /// Dokument-ID.
        doc: DocId,
        /// Inhalt.
        record: DocRecord,
    },
    /// Ersetzt ein vorhandenes Dokument; sonst ohne Wirkung.
    Update {
        /// Mandant.
        tenant: TenantId,
        /// Collection.
        collection: String,
        /// Dokument-ID.
        doc: DocId,
        /// Neuer Inhalt.
        record: DocRecord,
    },
    /// Retire: nicht mehr aktuell sichtbar, Historie bleibt.
    Delete {
        /// Mandant.
        tenant: TenantId,
        /// Collection.
        collection: String,
        /// Dokument-ID.
        doc: DocId,
    },
    /// `derived_from`-Kante; nur für vorhandene Dokumente.
    Link {
        /// Abgeleitetes Objekt.
        obj: ObjRef,
        /// Collection der Quelle.
        collection: String,
        /// Quelldokument.
        doc: DocId,
    },
    /// Checkpoint des aktuellen Stands.
    Checkpoint {
        /// Name (gleicher Name ersetzt).
        name: String,
    },
}

/// Ein beobachtbarer Eintrag in irgendeiner Schicht.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Entry {
    /// KV-Version (`None` = Tombstone), inklusive Historie.
    Version {
        /// KV-Schlüssel, siehe [`kv_key`].
        key: Vec<u8>,
        /// Sequenznummer.
        seq: u64,
        /// Wert.
        value: Option<Vec<u8>>,
    },
    /// Aktuell sichtbares Dokument (Text-/Vektorschicht).
    Doc {
        /// Mandant.
        tenant: TenantId,
        /// Collection.
        collection: String,
        /// Dokument-ID.
        doc: DocId,
        /// Inhalt.
        record: DocRecord,
    },
    /// Posting.
    Posting {
        /// Mandant.
        tenant: TenantId,
        /// Collection.
        collection: String,
        /// Term.
        term: String,
        /// Dokument-ID.
        doc: DocId,
    },
    /// Graphkante.
    Edge {
        /// Abgeleitetes Objekt.
        obj: ObjRef,
        /// Collection der Quelle.
        collection: String,
        /// Quelldokument.
        doc: DocId,
    },
    /// Checkpoint mit den enthaltenen Collections.
    Checkpoint {
        /// Name.
        name: String,
        /// Sequenz.
        seq: u64,
        /// Enthaltene `(Mandant, Collection)`.
        collections: BTreeSet<(TenantId, String)>,
    },
}

/// Vollständiger Bestand eines Systems.
pub type Inventory = BTreeSet<Entry>;

/// System unter Test (Implementierung im jeweiligen E2E-Test, O-X4).
pub trait SystemUnderTest {
    /// **Alles**, was in irgendeiner Schicht noch auffindbar ist, inklusive Historie.
    fn inventory(&self) -> Result<Inventory, String>;
    /// Über den Suchpfad aktuell sichtbare Dokumente.
    fn visible(&self, tenant: TenantId) -> Result<BTreeSet<(String, DocId)>, String>;
}

/// Abweichung zwischen Modell und System.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelDiff {
    /// Im Modell, aber nicht im System (beschädigte Nachbarn, Verlust).
    pub missing: Inventory,
    /// Im System, aber nicht im Modell (Wiederaufleben, Löschrest).
    pub unexpected: Inventory,
    /// Sichtbarkeitsabweichungen oder Fehler des Systems.
    pub notes: Vec<String>,
}

impl std::fmt::Display for ModelDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "model diff: missing={:?} unexpected={:?} notes={:?}",
            self.missing, self.unexpected, self.notes
        )
    }
}

impl std::error::Error for ModelDiff {}

/// Kollisionsfreier KV-Schlüssel eines Dokuments (längenpräfigierte Collection).
pub fn kv_key(tenant: TenantId, collection: &str, doc: DocId) -> Vec<u8> {
    let mut k = tenant.as_u64().to_be_bytes().to_vec();
    k.extend_from_slice(&(collection.len() as u64).to_be_bytes());
    k.extend_from_slice(collection.as_bytes());
    k.extend_from_slice(&u128::from(doc.0).to_be_bytes());
    k
}

type DocKey = (TenantId, String, DocId);

/// Sollzustand über alle Schichten. KV-Historie baut auf [`ReferenceModel`] auf.
#[derive(Debug, Clone, Default)]
pub struct ErasureModel {
    kv: ReferenceModel,
    docs: BTreeMap<DocKey, DocRecord>,
    postings: BTreeMap<(TenantId, String, String), BTreeSet<DocId>>,
    graph: BTreeMap<ObjRef, BTreeSet<(String, DocId)>>,
    checkpoints: BTreeMap<String, (u64, BTreeSet<(TenantId, String)>)>,
    /// Alle Schlüssel, die je zu einem KV-Eintrag gehörten (für `erase` nach `delete`).
    history_keys: BTreeMap<Vec<u8>, (DocKey, BTreeSet<String>)>,
}

impl ErasureModel {
    /// Leeres Modell.
    pub fn new() -> Self {
        Self::default()
    }

    fn unindex(&mut self, id: &DocKey) -> bool {
        let Some(old) = self.docs.remove(id) else {
            return false;
        };
        for term in old.terms {
            let key = (id.0, id.1.clone(), term);
            if let Some(set) = self.postings.get_mut(&key) {
                set.remove(&id.2);
                if set.is_empty() {
                    self.postings.remove(&key);
                }
            }
        }
        true
    }

    fn write(&mut self, id: DocKey, record: DocRecord) {
        self.unindex(&id);
        let key = kv_key(id.0, &id.1, id.2);
        self.history_keys
            .entry(key.clone())
            .or_insert_with(|| (id.clone(), BTreeSet::new()))
            .1
            .insert(record.key.clone());
        self.kv.put(&key, &record.body);
        self.kv.commit();
        for term in &record.terms {
            self.postings
                .entry((id.0, id.1.clone(), term.clone()))
                .or_default()
                .insert(id.2);
        }
        self.docs.insert(id, record);
    }

    /// Wendet eine Operation an.
    pub fn apply(&mut self, op: Op) {
        match op {
            Op::Insert {
                tenant,
                collection,
                doc,
                record,
            } => self.write((tenant, collection, doc), record),
            Op::Update {
                tenant,
                collection,
                doc,
                record,
            } => {
                let id = (tenant, collection, doc);
                if self.docs.contains_key(&id) {
                    self.write(id, record);
                }
            }
            Op::Delete {
                tenant,
                collection,
                doc,
            } => {
                let id = (tenant, collection, doc);
                if self.unindex(&id) {
                    self.kv.delete(&kv_key(id.0, &id.1, id.2));
                    self.kv.commit();
                }
            }
            Op::Link {
                obj,
                collection,
                doc,
            } => {
                if self
                    .docs
                    .contains_key(&(obj.tenant, collection.clone(), doc))
                {
                    self.graph.entry(obj).or_default().insert((collection, doc));
                }
            }
            Op::Checkpoint { name } => {
                let collections = self.docs.keys().map(|(t, c, _)| (*t, c.clone())).collect();
                self.checkpoints
                    .insert(name, (self.kv.snapshot_seq(), collections));
            }
        }
    }

    /// Entfernt das Ziel physisch aus **allen** Schichten, inklusive Historie.
    ///
    /// Checkpoints, die zum Zeitpunkt ihrer Erstellung ein betroffenes Dokument enthielten,
    /// werden verworfen (`CheckpointTaintPolicy::Drop`). Unbekannte Zielvarianten
    /// (`EraseTarget` ist `non_exhaustive`) löschen nichts und fallen im Vergleich auf.
    pub fn erase(&mut self, target: &EraseTarget) {
        let hit: BTreeMap<Vec<u8>, DocKey> = self
            .history_keys
            .iter()
            .filter(|(_, (id, user_keys))| match target {
                EraseTarget::Document {
                    tenant,
                    collection,
                    key,
                } => id.0 == *tenant && id.1 == *collection && user_keys.contains(key),
                EraseTarget::Collection { tenant, collection } => {
                    id.0 == *tenant && id.1 == *collection
                }
                EraseTarget::Tenant { tenant } => id.0 == *tenant,
                _ => false,
            })
            .map(|(k, (id, _))| (k.clone(), id.clone()))
            .collect();
        let tainted: Vec<String> = self
            .checkpoints
            .iter()
            .filter(|(_, (seq, _))| hit.keys().any(|k| self.kv.get_at(k, *seq).is_some()))
            .map(|(n, _)| n.clone())
            .collect();
        for name in tainted {
            self.checkpoints.remove(&name);
        }
        for (key, id) in &hit {
            self.unindex(id);
            // Nur Kanten desselben Mandanten: (Collection, DocId) ist nicht global eindeutig.
            for (_, sources) in self.graph.iter_mut().filter(|(o, _)| o.tenant == id.0) {
                sources.retain(|(c, d)| !(*c == id.1 && *d == id.2));
            }
            self.kv.erase_key(key);
            self.history_keys.remove(key);
        }
        self.graph.retain(|_, s| !s.is_empty());
    }

    /// Aktuell sichtbare Dokumente eines Mandanten.
    pub fn expected_visible(&self, tenant: TenantId) -> BTreeSet<(String, DocId)> {
        self.docs
            .keys()
            .filter(|(t, _, _)| *t == tenant)
            .map(|(_, c, d)| (c.clone(), *d))
            .collect()
    }

    /// Wert zum Stand `seq` (Zeitreise).
    pub fn get_at_seq(
        &self,
        tenant: TenantId,
        collection: &str,
        doc: DocId,
        seq: u64,
    ) -> Option<Vec<u8>> {
        self.kv.get_at(&kv_key(tenant, collection, doc), seq)
    }

    /// Aktuelle Sequenz.
    pub fn seq(&self) -> u64 {
        self.kv.snapshot_seq()
    }

    /// Soll-Bestand.
    pub fn inventory(&self) -> Inventory {
        let mut out = Inventory::new();
        for (key, seq, value) in self.kv.versions() {
            out.insert(Entry::Version {
                key: key.to_vec(),
                seq,
                value: value.map(<[u8]>::to_vec),
            });
        }
        for ((tenant, collection, doc), record) in &self.docs {
            out.insert(Entry::Doc {
                tenant: *tenant,
                collection: collection.clone(),
                doc: *doc,
                record: record.clone(),
            });
        }
        for ((tenant, collection, term), docs) in &self.postings {
            for doc in docs {
                out.insert(Entry::Posting {
                    tenant: *tenant,
                    collection: collection.clone(),
                    term: term.clone(),
                    doc: *doc,
                });
            }
        }
        for (obj, sources) in &self.graph {
            for (collection, doc) in sources {
                out.insert(Entry::Edge {
                    obj: obj.clone(),
                    collection: collection.clone(),
                    doc: *doc,
                });
            }
        }
        for (name, (seq, collections)) in &self.checkpoints {
            out.insert(Entry::Checkpoint {
                name: name.clone(),
                seq: *seq,
                collections: collections.clone(),
            });
        }
        out
    }

    /// Vergleicht Bestand und Sichtbarkeit aller bekannten und der angegebenen Mandanten.
    pub fn assert_matches<S: SystemUnderTest>(&self, sut: &S) -> Result<(), ModelDiff> {
        let mut diff = ModelDiff::default();
        let actual = match sut.inventory() {
            Ok(a) => a,
            Err(e) => {
                diff.notes.push(format!("inventory failed: {e}"));
                return Err(diff);
            }
        };
        let expected = self.inventory();
        diff.missing = expected.difference(&actual).cloned().collect();
        diff.unexpected = actual.difference(&expected).cloned().collect();
        let tenants: BTreeSet<TenantId> = expected
            .iter()
            .chain(&actual)
            .filter_map(|e| match e {
                Entry::Doc { tenant, .. } | Entry::Posting { tenant, .. } => Some(*tenant),
                Entry::Edge { obj, .. } => Some(obj.tenant),
                _ => None,
            })
            .collect();
        for tenant in tenants {
            match sut.visible(tenant) {
                Ok(v) if v == self.expected_visible(tenant) => {}
                other => diff
                    .notes
                    .push(format!("visibility of {tenant:?} differs: {other:?}")),
            }
        }
        if diff == ModelDiff::default() {
            Ok(())
        } else {
            Err(diff)
        }
    }
}
