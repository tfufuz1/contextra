# Agentic Context Pack: `contextra-ids`

> **Werkzeug:** `agentic_context_pack.py`  
> **Erstellungszeit:** `2026-11-18T17:59:13.922750+00:00`  
> **Status/Befunde:** `1 Befunde erfasst`  

## Beschreibung
Curated, minified context pack for coding agents assigned to `contextra-ids`. Optimized for LLM window density and context engineering.

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `crate_name` | `contextra-ids` |
| `ring` | `0` |
| `class` | `foundation` |

## 1. Architektur-Zuweisung
- **Crate:** `contextra-ids`
- **Ring:** `0`
- **Rolle/Klasse:** `foundation`
- **Pfad:** `crates/contextra-ids`

## 2. Crate-Spezifische AGENTS.md Verträge
# AGENTS.md — contextra-ids — Identitätstypen & Referenzen

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-ids.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-ids.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §2 (Architektur), §3.2 (Kapselung personenbezogener Daten), §4 (Schlüsselhierarchie), §13 (Invarianten)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E0 (Autarkes Fundament / Ring 0)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

Fundamentaler Mikro-Kernel für Identifikatoren, Mandanten-IDs, Referenzen und Sequenznummern.

---

## 2. Kern-Richtlinien für Agenten

- **Kanonischer Typ-Besitz:** Besitzt kanonisch `DocId`, `SubjectRef`, `PersonalRef`, `TenantId`, `CollectionId`, `TxId`, `ErasureId`, `TOMBSTONE_BIT`, `MAX_SEARCH_K`. Keine Duplikate in anderen Crates zulässig (R4).
- **Layout-Prüfung:** `size_of::<DocId>() == 8` (bzw. `16` unter Feature `docid-128`).
- **Sicherheits-Eigenschaften für `SubjectRef`:**
  - MUSS fehlbar über `rand::rngs::OsRng::try_fill_bytes` erzeugt werden.
  - KEIN `Clone` zulässig.
  - `ZeroizeOnDrop` zwingend implementiert.
  - Kein Klartext-`Display` (Verhinderung von Log-Leaks).
- **Mandanten-Isolation:** `TenantId(0)` ist fest für `TenantId::SYSTEM` reserviert. Reguläre IDs müssen > 0 sein.

## 3. Test-Pflichten
- T-ID1 bis T-ID6 (RNG-Erschöpfung, Zeroize bei Drop, BLAKE3-Präfix-Determinismus, TenantId(0)-Reservierung, CollectionId(0)-Ablehnung, Layout-Assertions).

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-ids` · **Ring:** – · **Version:** 0.1.0
- **Beschreibung:** Canonical identity and tenant types for the Contextra ecosystem
- **src:** 1 Dateien, 501 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 0):** –
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 3):** `blake3`, `serde`, `thiserror`
- **Reverse-Abhängigkeiten (8):** `contextra-checkpoint`, `contextra-crypto`, `contextra-graph`, `contextra-mvcc`, `contextra-store`, `contextra-text`, `contextra-types`, `contextra-vector`
- **Kopplung an Hub-Crates:** ausgehend –; eingehend `contextra-types`
- **Features (3):** `default`, `docid-128`, `std`

### Öffentliche API (grob, aus src/)
- struct (11): `CollectionId`, `DocId`, `DocRef`, `EntityId`, `FileId`, `Seq`, `StoreId`, `TenantId`, `TenantScoped`, `Timestamp`, `TxId`
- enum (2): `IdError`, `TenantScopeViolation`
- fn (21): `as_bytes`, `as_micros`, `as_u64`, `from_doc_id`, `from_key`, `from_micros`, `inner`, `internal`, `into_inner`, `into_inner_checked`, `is_internal`, `is_system`, `is_valid`, `is_valid_origin`, `map` … (+6)
- type (1): `Result`
- const (12): `DEFAULT`, `ID_WIDTH_BITS`, `INTERNAL_BASE`, `INVALID`, `MAX`, `MAX_COLLECTION_SEQUENCE`, `MAX_REF_LEN`, `MAX_SEARCH_K`, `MIN`, `SYSTEM`, `TOMBSTONE_BIT`, `ZERO`

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->

## 3. Normative Crate-Spezifikation (docs/crates/)
#
 
c
o
n
t
e
x
t
r
a
-
i
d
s
 
—
 
I
m
p
l
e
m
e
n
t
i
e
r
u
n
g
s
s
p
e
z
i
f
i
k
a
t
i
o
n
 
(
S
o
l
l
-
Z
u
s
t
a
n
d
)




*
*
E
b
e
n
e
:
*
*
 
E
0
 
(
A
u
t
a
r
k
e
s
 
F
u
n
d
a
m
e
n
t
 
/
 
R
i
n
g
 
0
)
 
 


*
*
K
e
r
n
-
C
l
o
s
u
r
e
:
*
*
 
L
e
e
r
 
(
0
 
i
n
t
e
r
n
e
 
W
o
r
k
s
p
a
c
e
-
A
b
h
ä
n
g
i
g
k
e
i
t
e
n
)
 
 


*
*
S
t
a
t
u
s
:
*
*
 
N
o
r
m
a
t
i
v
 
(
S
o
l
l
-
Z
u
s
t
a
n
d
 
f
ü
r
 
E
n
t
w
i
c
k
l
e
r
)
 
 


*
*
V
e
r
s
i
o
n
:
*
*
 
2
.
0
 
 


*
*
G
e
l
t
u
n
g
s
b
e
r
e
i
c
h
:
*
*
 
`
c
r
a
t
e
s
/
c
o
n
t
e
x
t
r
a
-
i
d
s
`
 
 


*
*
R
e
f
e
r
e
n
z
d
o
k
u
m
e
n
t
e
:
*
*
 
`
d
o
c
s
/
S
P
E
C
.
m
d
`
 
§
2
,
 
§
3
.
2
,
 
§
4
,
 
§
1
3
;
 
`
A
N
A
L
Y
S
I
S
-
E
0
-
C
O
N
T
R
A
D
I
C
T
I
O
N
S
.
m
d
`
 
(
D
-
0
1
,
 
D
-
0
6
)
;
 
`
A
L
T
L
A
S
T
E
N
-
A
U
D
I
T
.
m
d
`
 
 




-
-
-




#
#
 
1
.
 
Z
w
e
c
k
 
&
 
A
r
c
h
i
t
e
k
t
u
r
g
a
r
a
n
t
i
e
n




`
c
o
n
t
e
x
t
r
a
-
i
d
s
`
 
i
s
t
 
d
e
r
 
f
u
n
d
a
m
e
n
t
a
l
e
 
M
i
k
r
o
-
K
e
r
n
e
l
 
f
ü
r
 
I
d
e
n
t
i
f
i
k
a
t
o
r
e
n
,
 
M
a
n
d
a
n
t
e
n
-
I
D
s
,
 
R
e
f
e
r
e
n
z
e
n
 
u
n
d
 
M
o
n
o
t
o
n
i
e
-
T
y
p
e
n
 
d
e
s
 
g
e
s
a
m
t
e
n
 
C
o
n
t
e
x
t
r
a
-
Ö
k
o
s
y
s
t
e
m
s
.




#
#
#
 
A
r
c
h
i
t
e
k
t
u
r
g
a
r
a
n
t
i
e
n
 
(
R
1
–
R
6
,
 
I
N
V
-
3
,
 
I
N
V
-
4
,
 
I
N
V
-
1
1
)


1
.
 
*
*
0
 
i
n
t
e
r
n
e
 
W
o
r
k
s
p
a
c
e
-
A
b
h
ä
n
g
i
g
k
e
i
t
e
n
:
*
*
 
`
c
o
n
t
e
x
t
r
a
-
i
d
s
`
 
h
ä
n
g
t
 
v
o
n
 
k
e
i
n
e
m
 
a
n
d
e
r
e
n
 
C
o
n
t
e
x
t
r
a
-
C
r
a
t
e
 
a
b
.


2
.
 
*
*
`
#
!
[
f
o
r
b
i
d
(
u
n
s
a
f
e
_
c
o
d
e
)
]
`
:
*
*
 
1
0
0
 
%
 
s
i
c
h
e
r
e
s
 
R
u
s
t
,
 
k
e
i
n
e
 
A
u
s
n
a
h
m
e
n
.


3
.
 
*
*
Z
e
r
o
-
P
a
n
i
c
-
P
o
l
i
c
y
:
*
*
 
K
e
i
n
e
 
V
e
r
w
e
n
d
u
n
g
 
v
o
n
 
`
u
n
w
r
a
p
(
)
`
,
 
`
e
x
p
e
c
t
(
)
`
,
 
`
p
a
n
i
c
!
(
)
`
,
 
`
t
o
d
o
!
(
)
`
 
i
m
 
N
i
c
h
t
-
T
e
s
t
-
C
o
d
e
.


4
.
 
*
*
L
e
e
r
e
 
D
e
f
a
u
l
t
-
F
e
a
t
u
r
e
s
 
(
R
3
)
:
*
*
 
`
d
e
f
a
u
l
t
 
=
 
[
]
`
.
 
F
e
a
t
u
r
e
s
 
w
i
e
 
`
d
o
c
i
d
-
1
2
8
`
 
o
d
e
r
 
`
s
t
d
`
 
s
i
n
d
 
o
p
t
-
i
n
.


5
.
 
*
*
E
i
n
 
K
o
n
z
e
p
t
 
—
 
e
i
n
 
B
e
s
i
t
z
e
r
 
(
R
4
)
:
*
*
 
`
c
o
n
t
e
x
t
r
a
-
i
d
s
`
 
i
s
t
 
d
e
r
 
*
*
a
l
l
e
i
n
i
g
e
 
k
a
n
o
n
i
s
c
h
e
 
B
e
s
i
t
z
e
r
*
*
 
f
ü
r
:


 
 
 
-
 
`
D
o
c
I
d
`
,
 
`
E
n
t
i
t
y
I
d
`
,
 
`
T
e
n
a
n
t
I
d
`
,
 
`
C
o
l
l
e
c
t
i
o
n
I
d
`
,
 
`
T
x
I
d
`


 
 
 
-
 
`
S
u
b
j
e
c
t
R
e
f
`
,
 
`
P
e
r
s
o
n
a
l
R
e
f
`
,
 
`
T
e
n
a
n
t
R
e
f
`
,
 
`
C
o
l
l
e
c
t
i
o
n
R
e
f
`
,
 
`
O
p
e
r
a
t
o
r
R
e
f
`
,
 
`
D
o
c
K
e
y
`
,
 
`
D
o
c
R
e
f
`


 
 
 
-
 
`
E
r
a
s
u
r
e
I
d
`
,
 
`
G
a
t
e
I
d
`
,
 
`
F
i
l
e
I
d
`
,
 
`
S
t
o
r
e
I
d
`
,
 
`
S
e
q
`
,
 
`
T
i
m
e
s
t
a
m
p
`


 
 
 
-
 
`
T
e
n
a
n
t
S
c
o
p
e
d
<
T
>
`
,
 
`
T
e
n
a
n
t
S
c
o
p
e
V
i
o
l
a
t
i
o
n
`
,
 
`
I
d
E
r
r
o
r
`


 
 
 
*
(
A
l
l
e
 
K
o
p
i
e
n
 
u
n
d
 
R
e
-
E
x
p
o
r
t
s
 
i
n
 
`
t
y
p
e
s
`
,
 
`
p
r
i
v
a
c
y
`
,
 
`
e
r
a
s
u
r
e
`
 
w
e
r
d
e
n
 
d
u
r
c
h
 
d
i
e
s
e
n
 
K
a
n
o
n
 
e
r
s
e
t
z
t
)
.
*




-
-
-




#
#
 
2
.
 
K
a
n
o
n
i
s
c
h
e
 
T
y
p
d
e
f
i
n
i
t
i
o
n
e
n
 
&
 
S
p
e
i
c
h
e
r
g
r
e
n
z
e
n




#
#
#
 
2
.
1
 
Ü
b
e
r
s
i
c
h
t
 
d
e
r
 
B
i
t
b
r
e
i
t
e
n
 
&
 
S
e
m
a
n
t
i
k
e
n




`
`
`


┌
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
┐


│
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
c
o
n
t
e
x
t
r
a
-
i
d
s
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
│


│
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
│


│
 
 
 
┌
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
┐
 
 
 
 
 
 
 
 
 
 
 
 
 
┌
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
─
┐
 
 
 
│


│
 
 
 
│
 
 
 
 
 
 
 
 
 
 
D
o
c
I
d
 
 
 
 
 
 
 
 
 
 
 
│
 
 
 
 
 
 
 
 
 
 
 
 
 
│
 
 
 
 
 
 
 
 
 
S
u
b
j
e
c
t
R
e
f
 
 
 
 
 
 
 
 
 
│
 
 
 
│


│
 
 
 
│
(
6
4
-
B
i
t
 
/
 
1
2
8
-
B
i
t
 
B
L
A
K
E
3
)
 
│
 
 
 
 
 
 
 
 
 
 
 
 
 
│
 
(
1
2
8
-
B
i
t
 
P
I
I
-
Z
u
f
a
l
l
,
 
Z
e
r
o
)
 
│
 
 
 
│

## 4. Öffentliche Schnittstellen & Signaturen (lib.rs)
```rust
pub const ID_WIDTH_BITS: u32 = if cfg!(feature =            ) { 128 } else { 64 };
pub const TOMBSTONE_BIT: u64 = 1 << 63;
pub const MAX_SEARCH_K: usize = 1_000;
pub const MAX_REF_LEN: usize = 256;
pub enum IdError {
pub type Result<T> = core::result::Result<T, IdError>;
pub struct TenantId(pub u64);
pub const SYSTEM: Self = Self(0);
pub const DEFAULT: Self = Self(0);
pub const INVALID: Self = Self(0);
pub const fn new(id: u64) -> Self {
pub fn try_new(id: u64) -> Result<Self> {
pub const fn inner(self) -> u64 {
pub const fn as_u64(self) -> u64 {
pub const fn is_system(self) -> bool {
pub struct CollectionId(pub u64);
pub const SYSTEM: Self = Self(0);
pub const INVALID: Self = Self(0);
pub const fn new(id: u64) -> Self {
pub fn try_new(id: u64) -> Result<Self> {
pub const fn inner(self) -> u64 {
pub const fn as_u64(self) -> u64 {
pub struct DocId(pub u64);
pub struct DocId(pub u128);
pub const MAX: Self = Self(u64::MAX);
pub const MIN: Self = Self(0);
pub const fn new(id: u64) -> Self {
pub fn from_key(key: &str) -> Result<Self> {
pub const fn inner(self) -> u64 {
pub const fn as_u64(self) -> u64 {
pub const MAX: Self = Self(u128::MAX);
pub const MIN: Self = Self(0);
pub const fn new(id: u128) -> Self {
pub fn from_key(key: &str) -> Result<Self> {
pub const fn inner(self) -> u128 {
pub const fn as_u64(self) -> u64 {
pub struct EntityId(pub u64);
pub const fn new(id: u64) -> Self {
pub const fn inner(self) -> u64 {
pub const fn as_u64(self) -> u64 {
pub fn as_bytes(&self) -> Vec<u8> {
pub fn from_doc_id(doc_id: DocId) -> Self {
pub fn from_doc_id(doc_id: DocId) -> Self {
pub fn from_key(key: &str) -> Result<Self> {
pub struct TxId(pub u64);
pub const INVALID: Self = Self(0);
pub const MAX_COLLECTION_SEQUENCE: u64 = 1_000_000_000_000;
pub const INTERNAL_BASE: u64 = u64::MAX - 1_000_000;
pub const fn new(id: u64) -> Self {
pub const fn inner(self) -> u64 {
```

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra-ids` | `crates/contextra-ids/Cargo.toml` | `1` | Generated focused context pack for contextra-ids |

---
*Automatisch generiert durch `agentic_context_pack.py` für Agentic Engineering & Context Auditing.*