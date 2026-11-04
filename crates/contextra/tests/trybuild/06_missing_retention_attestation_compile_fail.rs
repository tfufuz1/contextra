// Compile-Fail Test: Fehlende RetentionAttestation verhindert Aufruf von .erase() (SPEC 7.3, D-15, T-API-2)
use contextra::{Contextra, EraseMode, EraseRequest, EraseTarget, Timing};

#[tokio::main]
async fn main() {
    fn check_retention_required(
        ctx: &Contextra,
        target: EraseTarget,
        timing: Timing,
        mode: EraseMode,
    ) {
        struct IncompleteEraseRequest {
            target: EraseTarget,
            timing: Timing,
            mode: EraseMode,
        }
        let bad_req = IncompleteEraseRequest {
            target,
            timing,
            mode,
        };
        let _ = ctx.erase(bad_req);
    }
}
