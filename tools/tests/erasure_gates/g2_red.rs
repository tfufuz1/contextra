fn a() {
    let _ = remove(); //~
    let _: Result<(), E> = remove(); //~
    let v = read().unwrap_or_default(); //~
    sync().ok(); //~
    let _ = remove(); // gate-allow: G-2 //~
}
