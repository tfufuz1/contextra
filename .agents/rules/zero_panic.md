# Keine Panics, keine verschluckten Fehler

Abgeleitet aus P2, P6, SPEC 8.13, 5.8, I-11, D-87.

- Im Produktionscode gibt es kein `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, `unimplemented!` und kein ungeprüftes Indexieren oder Slicen.
- `let _ = …` auf ein `Result` ist verboten. Fehler werden propagiert oder als typisierter Fehler des Crates zurückgegeben.
- `unwrap_or_default` auf I/O-Ergebnissen ist verboten.
- Sentinel-Werte als Ersatz für `Option` sind verboten (z. B. `u64::MAX` als „alles sichtbar“, I-12).
- Fehlertexte sind fest und enthalten keine Nutzerinhalte, Schlüssel oder Klartext-Kennungen (I-11, HR-4).
- Tests dürfen `unwrap` und `expect` verwenden, wenn der Test sonst nicht lesbar wäre. Die Ausnahme gilt nicht für Produktionscode.
- Der Ratchet in `governance/ratchet.toml` erlaubt nur sinkende Zahlen. Ziel ist null in allen Kern-Crates ab Gate G-C.
