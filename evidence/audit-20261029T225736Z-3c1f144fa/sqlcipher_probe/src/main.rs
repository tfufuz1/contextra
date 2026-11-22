
use rusqlite::Connection;
fn main() {
    let path = std::env::args().nth(1).expect("path");
    let _ = std::fs::remove_file(&path);
    {
        let c = Connection::open(&path).unwrap();
        c.execute_batch("PRAGMA key='probe-key-1';").unwrap();
        c.execute_batch("CREATE TABLE t(id INTEGER PRIMARY KEY, body TEXT); INSERT INTO t(body) VALUES ('KLARTEXT_MARKER_4711');").unwrap();
        let v: Result<String, _> = c.query_row("PRAGMA cipher_version", [], |r| r.get(0));
        println!("cipher_version={:?}", v.ok());
        let fts = c.execute_batch("CREATE VIRTUAL TABLE d USING fts5(x); INSERT INTO d(x) VALUES ('Rechnung Mueller');");
        println!("fts5={}", fts.is_ok());
        let sd: Result<i64, _> = c.query_row("PRAGMA secure_delete=ON", [], |r| r.get(0));
        println!("secure_delete={:?}", sd.ok());
    }
    let bytes = std::fs::read(&path).unwrap();
    let leak = bytes.windows(20).any(|w| w == b"KLARTEXT_MARKER_4711");
    println!("plaintext_leak={}", leak);
    {
        let c = Connection::open(&path).unwrap();
        c.execute_batch("PRAGMA key='falsch';").unwrap();
        let wrong = c.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get::<_, i64>(0));
        println!("wrong_key_rejected={}", wrong.is_err());
    }
    let c = Connection::open(&path).unwrap();
    c.execute_batch("PRAGMA key='probe-key-1';").unwrap();
    let rows: i64 = c.query_row("SELECT count(*) FROM t", [], |r| r.get(0)).unwrap_or(-1);
    println!("right_key_rows={}", rows);
}
