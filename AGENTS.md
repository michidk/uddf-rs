# Agent instructions

`uddf` is a Rust library that reads and writes UDDF (Universal Dive Data Format) 3.2.x dive
logs. Licensed MIT OR Apache-2.0; it must stay free of GPL and LGPL code.

## Layout

- `src/model/`: the typed model, one file per area (`common`, `people`, `equipment`,
  `places`, `dive`). `record!` and `vocabulary!` in `model/mod.rs` add the shared derives,
  `#[non_exhaustive]` and the serde wiring.
- `src/read.rs`: lenient `roxmltree` reader. `src/write.rs`: `quick-xml` writer in schema
  order. `src/refs.rs`: id resolution. `src/datetime.rs`: the lossless `DateTime`.
- `tests/fixtures.rs` with vendored files in `tests/fixtures/` (see its README) and insta
  snapshots in `tests/snapshots/`.
- `.github/workflows/`: `ci.yml`, `pr-title.yml`, `bump-version.yml`, `release.yml` and
  `publish-cratesio.yml` (and `cliff.toml`) are managed by Orcastrate from
  `michidk/workflows`; change only their frontmatter `params`. `checks.yml` (WebAssembly
  build, `cargo-deny`) is repository-owned.

## Rules

- Write from the UDDF specification and real files. Do not read, copy or depend on GPL or
  LGPL code (including the `uddf-sdk` crate).
- Reading never invents values: absent stays `None`, values keep UDDF's SI units and the
  file's own spelling. Every new field documents its UDDF element and unit.
- A value that cannot be parsed is an `InvalidValue`, never a panic or a silent default.
- The writer follows the XSD's element order. Every new element is read, written and
  covered by the round-trip test.
- No `unwrap`, `expect`, `panic` or indexing in library code; strict clippy lints apply.
- Must build for `wasm32-unknown-unknown`.
- Commits and PR titles follow Conventional Commits (`feat(read): ...`, `fix(write): ...`).
  Work on a feature branch; do not push to `main`.

## Checks

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo clippy --target wasm32-unknown-unknown --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo deny check
```

Use the machine's shared Cargo target directory; never pass `--target-dir`.
