# SP-CLUSTER — Cluster, Spaces, Links, Restrictions, Packgruppen

| | |
|---|---|
| Modul | `erase/{cluster_table,links,restrictions}.rs`, `spaces.rs`, `engine/packgroups.rs` |
| Besitzt | Cluster-/Space-Lebenszyklus, Kern-ID-Erzeugung, Schutzklasse, Links, Claims, Restrictions, Clusterauflistung |
| Konsumiert | PORTS, ENV, CATALOG, COMMIT, KEYS |
| Spec-Bezug | R8 §4, §5, §9.6, §12.7; INV-ID-1/2, INV-CLAIM-1, INV-PL-1…5, INV-SEAL-1, INV-ERA-10 |
| Phase | P1 Basis, P6 vollständig (G-E) |

## 1 Auftrag
Cluster sind Lösch-, Schlüssel-, Versions- und Platzierungsdomäne. Nur der Kern erzeugt ClusterId aus Entropy. `ClusterSpec{protection, pack, tag}` und `Clusters::list(filter)` schließen die Auffindungslücke E-33; Tag ≤32 B, Sealed-Tag ausschließlich im versiegelten SYS-Header. Cluster-Tabelle kompakt ≤48 B/Eintrag; Branch-Tabellen lazy (E-38).

## 2 Verträge
`create_cluster`, `list`, `link/unlink`, `restrict/lift`, `space_create/drop`. `link(derived,sources[<=8])` hält einen azyklischen `DerivedFrom`-Graph. Jede mögliche transitive Erase-Hülle ist auf `max_gate_clusters=4096` begrenzt: Admission berechnet sie auf einer Graph-Generation außerhalb Rang 1; Commit validiert Generation und Hülle erneut. Überschreitung ergibt `EraseFanoutExceeded` ohne Teilmutation. Fachliche Gleichheit der Personen-/Löschdomäne prüft die obere Identity-/Policy-Schicht; der Storage-Kern kennt keine Personenidentität. Link-Hülle trägt BLOCK_ERASE-Restriction. Claim gehört einem Handle und begrenzt jede Dateioperation. Schutzklasse Sealed/Plain wird beim Erzeugen festgelegt; `personal ⇒ Sealed`.

## 3 Regeln
Keine ID-Wiederverwendung. Link-Kaskade und Restriction werden im Committer-Frame geprüft. Lift folgt `LiftPolicy`; keine Override-Möglichkeit für BLOCK_ERASE. Packgruppe enthält ausschließlich kompatible Cluster; Pure/Packed/Staging-Regeln und Levels werden vor Catalog-Edit validiert. Cluster dürfen nicht aus Caller-ID oder Kennung ableitbar sein.

## 4 Invarianten / Umsetzung
INV-ID-1/2, INV-CLAIM-1, INV-PL-1…5, INV-SEAL-1, INV-ERA-10. Tests: Restart/ID-Reserve, Claim-Fremdedit-Abweisung, Restriction während Link-Kaskade, Packgruppenmodell; keine Klartext-Personenmetadaten.

## 5 Arbeitspakete
WP-12.1 Register/ClusterSpec/list (P1) · WP-12.2 Links + transitiver Abschluss (P6) · WP-12.3 Restrictions/LiftPolicy (P6) · WP-12.4 Packgruppenregeln + Claim (P6). Abnahme: Restriction nicht umgehbar; Erase-Abschluss stabil zum Gate-Zeitpunkt.
