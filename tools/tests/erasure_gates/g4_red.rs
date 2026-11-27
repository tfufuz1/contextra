use std::fs; //~
use std::{io, fs::File}; //~

async fn a() {
    tokio::fs::remove_file("x").await; //~
}
