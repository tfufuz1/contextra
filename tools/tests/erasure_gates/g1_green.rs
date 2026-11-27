//! block_on in Doku ist erlaubt.
async fn a() {
    /* block_on */
    log("block_on");
    work().await;
}

#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        rt().block_on(super::a());
    }
}
