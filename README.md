# ads-bazaar-contract

Soroban (Stellar) smart contracts for **AdsBazaar** — a decentralized
marketplace for multi-currency creator campaigns.

AdsBazaar lets businesses fund influencer marketing campaigns in the
currency they already use, while creators receive escrow-protected payouts
through Stellar assets and local payment rails. The initial focus is
emerging-market creator commerce: Nigerian businesses paying in
Naira-denominated assets, Kenyan creators withdrawing through
mobile-money-connected anchors, and global teams settling campaigns in
stablecoins — all without rebuilding the same trust and FX workflow for
every country.

This repository is the on-chain layer: the escrow that holds campaign
budgets and the arbitration flow for contested payouts.

> **Status: early scaffold.** The contract data model, storage schema, error
> types, and public API surface are in place; the state-transition logic for
> most of the marketplace flow is intentionally left as `todo!()` for
> contributors. See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for
> exactly what's implemented vs. open, and
> [`CONTRIBUTING.md`](CONTRIBUTING.md) for how to pick up a piece of it.

## Structure

```
contracts/
├── shared/               common types (CampaignStatus, PayoutAsset, ...)
├── campaign-escrow/      holds and releases campaign budgets
└── dispute-resolution/   arbitrates contested creator payouts
docs/
└── ARCHITECTURE.md       design overview + open questions
```

## Requirements

- [Rust](https://rustup.rs/) — toolchain and wasm targets are pinned in
  `rust-toolchain.toml`
- [Stellar CLI](https://developers.stellar.org/docs/tools/cli/install-cli)
  (`stellar`) for building and deploying contracts

## Quick start

```sh
# run tests
cargo test --workspace

# lint + format
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

# build deployable .wasm for every contract
stellar contract build
```

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). The short version: read
`docs/ARCHITECTURE.md`, pick a `todo!()`, and open a PR — most stubs have an
open design question attached that's worth discussing before implementing.

## License

[MIT](LICENSE)
