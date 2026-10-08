# Contributing to ads-bazaar-contract

Thanks for helping build AdsBazaar's on-chain layer. The escrow lifecycle is
implemented and tested. What's left is mainly arbiter-driven dispute
resolution plus a few smaller design questions (see `docs/ARCHITECTURE.md`
for the current state and the open design questions).

## Getting set up

You'll need:

- [Rust](https://rustup.rs/) (the toolchain and `wasm32v1-none` /
  `wasm32-unknown-unknown` targets are pinned in `rust-toolchain.toml` and
  will be installed automatically by `rustup` on first use)
- The [Stellar CLI](https://developers.stellar.org/docs/tools/cli/install-cli)
  (`stellar`) for building/deploying/invoking contracts against a local or
  test network

```sh
cargo test --workspace          # run all unit tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all                 # or `-- --check` in CI
stellar contract build          # builds every contract to .wasm
```

## Where to start

1. Read `docs/ARCHITECTURE.md` — it lists exactly what's implemented, what's
   still `todo!()`, and the open design questions.
2. Pick an open item: one of the two remaining `todo!()`s
   (`dispute-resolution::resolve_dispute`,
   `campaign-escrow::resolve_dispute_payout`), a `TODO(contributors)` doc
   comment, or an open design question. Each `todo!()` has a doc comment
   directly above it describing the intended behavior.
3. If the design question it depends on isn't settled yet, open an issue or
   discussion proposing an approach before writing the implementation —
   these are the decisions other contributors will build on top of.
4. Add tests alongside your implementation in the crate's `test.rs`, and in
   `contracts/campaign-escrow/tests/integration.rs` for anything that crosses
   the escrow ↔ dispute-resolution boundary. The existing tests show the
   pattern: register the contract, `env.mock_all_auths()`, call through the
   generated `Client`.

## Pull requests

- Keep PRs scoped to one `todo!()` / one design question where possible —
  it's much easier to review and merge incrementally than as one large
  change.
- Make sure `cargo fmt --all -- --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, and `cargo test --workspace` all pass (CI
  runs the same checks).
- Explain *why* in the PR description, especially for anything touching an
  open design question — the reasoning matters as much as the code for
  contract logic that moves real funds.

## Code style

- No comments explaining *what* the code does — name things clearly
  instead. Comments are for *why*: a non-obvious constraint, an invariant,
  a reason a simpler approach doesn't work.
- Prefer extending an existing `Error` variant over introducing a new
  pattern for the same failure mode.
- Escrow-affecting logic (anything that moves or releases funds) should
  favor explicit checks and `Result<_, Error>` returns over `panic!`/
  `unwrap()` — the exception is the `todo!()` stubs themselves, which are
  expected to panic until implemented.
