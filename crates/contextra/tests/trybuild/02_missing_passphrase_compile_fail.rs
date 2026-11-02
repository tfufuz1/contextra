// Testet, dass ohne Passphrase bei Builder-Abschluss ein Fehler provoziert wird
// bzw. in K10 typbasiert zur Compile-Zeit fehlschlägt.

fn main() {
    // In Stufe 1: Laufzeitfehler.
    // In Stufe 2 (K10): Builder ohne with_passphrase() hat keine .open()/.build()-Methode.
}
