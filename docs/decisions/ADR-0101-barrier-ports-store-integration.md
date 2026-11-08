# ADR-0101: Abgrenzung der Barrieren-Ports (CommitBarrier / PurgeBarrier) zwischen contextra-store und contextra-engine / contextra-erasure

- **Status**: Akzeptiert
- **Datum**: 2026-11-08
- **Kontext**: SPEC §3.5, SPEC §11.3, §11.4, D-57, D-78, `contextra-ports/src/barrier.rs`

## Kontext und Problemstellung
In `contextra-ports/src/barrier.rs` sind die Schnittstellen `CommitBarrier` und `PurgeBarrier` definiert:
- `CommitBarrier`: Kontrolliert die Synchronisation zwischen konkurrierenden Transaktions-Commits (`shared`) und exklusiven Lösch-Gate-Zustandsübergängen (`exclusive`).
- `PurgeBarrier`: Kontrolliert den wechselseitigen Ausschluss zwischen Hintergrund-LSM-Kompaktierungen (`compaction`) und physischen Lösch-Purges (`purge`).

Es stellte sich die architektonische Frage, wo diese Barrieren implementiert werden und wie `contextra-store` daran angebunden wird:
Soll `contextra-store` die Traits implementieren, oder leben sie auf übergeordneter Ebene (`contextra-engine` bzw. `contextra-erasure`)?

## Analyse & Abgleich

1. **Aufgabenbereich von `contextra-store` (Ring 1)**:
   - `contextra-store` ist eine reine Persistenz- und LSM-Tree-Engine (MemTable, SSTables, WAL, lokaler Flush & Compaction).
   - Gemäß Baukasten-Architektur (SPEC v4.0, M7) darf der Store **keine Lösch-Saga-, GDPR- oder Proof-Logik** enthalten.
   - Der Store benötigt lokalen Fencing-Schutz zwischen seinem eigenen `flush()` und bereichsweisen physischen `purge()`-Aufrufen (`LsmPurger`). Dies wird storespezifisch über `FlushPurgeFence` (Dekker-Muster `SeqCst`) realisiert.

2. **Aufgabenbereich der Barrieren-Ports (`contextra-ports`)**:
   - `CommitBarrier` und `PurgeBarrier` koordinieren systemweite Phasen über mehrere Teilsysteme hinweg (Vektorspeicher, Textindex, Graph, relationaler Store, Vault).
   - Die Initiatoren exklusiver Phasen sind der Löschkoordinator (`ErasureCore` / `GateManager` in `contextra-erasure`).
   - Die Bereitstellung der gemeinsamen Barrieren-Instanz obliegt der orchestrationsfähigen Ebene (`contextra-engine`), welche die Teilspeicher instanziiert und verdrahtet.

## Entscheidung

1. **Keine Direkt-Implementierung der globalen Barrieren-Traits in `contextra-store`**:
   - `contextra-store` implementiert `CommitBarrier` und `PurgeBarrier` **nicht** als primäre Instanz.
   - Grund: Ein LSM-Store kennt weder Erasure-Sagas noch globale Gate-Phasen. Eine Implementierung im Store würde Ring-1-Code an Ring-3-Spezifika binden oder unvollständige Stubs erfordern.

2. **Saubere Kapselung der Store-Seite**:
   - `contextra-store` stellt lokales Write-Fencing über `src/lsm/fence.rs` (`FlushPurgeFence`) bereit.
   - Wenn `contextra-engine` oder `contextra-erasure` einen physischen Purge oder Gate-Schritt ausführen, erwerben sie über die übergeordnete Barriere das Permit und rufen die Ports des Stores (`StorageEngine`, `Purger`) auf.
   - Dadurch bleiben Crate-Grenzen strikt entkoppelt (INV-LAYER-1, D-E1).

## Konsequenzen
- **Positiv**: `contextra-store` bleibt ein unabhängiger, wiederverwendbarer Lego-Baustein ohne Wissen über höhere Lösch-Orchestrierung.
- **Positiv**: Keine unvollständigen Zwischenlösungen oder zyklischen Kopplungen im Workspace.
- **Neutral**: `contextra-engine` bindet die Barrieren-Ports an und koordiniert Zugriffe auf Store, Vektorindex und Erasure-Saga.
