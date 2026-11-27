fn a() -> Result<(), E> {
    let _guard = lock();
    let v = read().ok().map(use_it);
    let _ = remove(); // gate-allow: G-2 Datei ist bereits entfernt, Ergebnis irrelevant
    let s = "let _ = x; .ok();";
    sync()?;
    Ok(())
}
