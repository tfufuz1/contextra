# Fuzz Smoke Gate

## Overview
The `fuzz-smoke` gate runs cargo fuzz targets for touched crates containing fuzz targets for a bounded duration (default 60s per target) to detect runtime panics or crashes introduced in pull requests.

## Usage
```bash
cargo xtask fuzz-smoke [--root <dir>] [--json] [--secs <secs>] [--base <rev>] [--head <rev>]
```
