# Architecture

AdsBazaar's on-chain layer is a Cargo workspace of Soroban (Stellar smart
contracts) crates:

```
contracts/
├── shared/               ads-bazaar-shared     — common types/enums, no #[contract]
├── campaign-escrow/      ads-bazaar-campaign-escrow — holds & releases campaign funds
└── dispute-resolution/   ads-bazaar-dispute-resolution — arbitrates contested payouts
```

`shared` has no contract entry points of its own — it exists purely so the
two contracts (and any future ones) agree on the same `CampaignStatus`,
`ApplicationStatus`, `DisputeStatus`, `DisputeOutcome` and `PayoutAsset`
types instead of drifting apart.

## Why two contracts instead of one

- **Escrow correctness is the highest-stakes code in this repo** — it holds
  real business funds. Keeping it free of arbitration logic keeps its
  surface area (and audit surface) as small as possible.
- **Arbitration is the least-settled design space.** Whether disputes are
  resolved by a single trusted arbiter, a staked jury, or an oracle is an
  open question (see `dispute-resolution/src/lib.rs`). Isolating it in its
  own contract means that design can iterate — or even be redeployed —
  without touching escrow.
- Cross-contract calls between them go through an explicit, narrow
  interface: `campaign-escrow::freeze_for_dispute` /
  `resolve_dispute_payout`, callable only by the configured
  `dispute_contract` address, and `dispute-resolution` calling back into
  escrow once a dispute resolves. `freeze_for_dispute` is implemented (a
  per-application freeze wired up by `raise_dispute`), as is the reverse
  `dispute-resolution::close_dispute` callback that escrow's admin
  `resolve_dispute` uses to close the dispute record. The arbiter-driven
  `resolve_dispute_payout` is still `todo!()`.

### The evidence window

`freeze_for_dispute` is the point at which escrow learns a dispute exists: it
stamps `Application::dispute_opened_at` and emits `DisputeFrozen`. The
admin-resolved shortcut `campaign-escrow::resolve_dispute` will not settle an
application that has no such stamp, and will not settle one until
`MIN_EVIDENCE_WINDOW` (72h) has elapsed since it.

This is a deliberate trust-minimization bound on the admin key rather than an
implementation detail. `resolve_dispute` can move a committed payout in any
direction, so without the window a compromised admin key could reallocate a
disputed payout in the same block the dispute is raised — before the other
party could answer it, or even see it. Requiring the freeze first is what
makes the window meaningful: it guarantees a public `DisputeFrozen` event
precedes every settlement, so there is something for the counterparty to
notice. The constant is `pub` so a frontend can render the countdown instead
of duplicating the number.

## Multi-currency design

A campaign is funded in whatever Stellar asset the business already holds —
represented by `PayoutAsset { token: Address, symbol: String }`, where
`token` is any SEP-41-compatible token contract address (a classic Stellar
Asset Contract wrapping XLM/a Naira-pegged stablecoin/USDC, or a native
Soroban token). The escrow contract is asset-agnostic: it's expected to move
funds via the standard `soroban_sdk::token::Client` rather than special-
casing any one currency. This is what lets a Lagos business fund a campaign
in a Naira-denominated asset and a Nairobi creator withdraw through a
mobile-money-connected anchor without either side touching a different
contract.

## Current state

The full escrow lifecycle is implemented and tested. The only remaining
`todo!()`s are the two halves of arbiter-driven dispute resolution:

| Area | Status |
|---|---|
| Storage schema, error types, event types (all events published) | Implemented |
| `initialize` (both contracts) | Implemented |
| `create_campaign`, `fund_campaign`, `update_campaign_metadata` | Implemented |
| `apply_to_campaign`, `approve_creator`, `submit_proof` | Implemented |
| `approve_submission`, `reject_submission`, `claim_payment` | Implemented |
| `cancel_campaign`, `expire_campaign`, `reclaim_surplus`, `emergency_recover_campaign` | Implemented |
| Admin controls: `pause`/`unpause`, `propose_admin`/`accept_admin`, `update_fee_bps`, `update_treasury`, `upgrade` | Implemented |
| `freeze_for_dispute`, escrow admin `resolve_dispute` (after `MIN_EVIDENCE_WINDOW`) | Implemented |
| `raise_dispute`, `assign_arbiter`, `close_dispute` (dispute-resolution) | Implemented |
| Read-only getters (`get_campaign`, `get_application`, `applicant_count`, `campaign_applicants`, `get_campaign_business`, `get_protocol_config`, `get_dispute`, `version`) | Implemented |
| `resolve_dispute` (dispute-resolution) | `todo!()` |
| `resolve_dispute_payout` (campaign-escrow) | `todo!()` |

Both `todo!()`s have a doc comment directly above them, and both depend on the
arbitration-model question below.

## Testing strategy

Testing is split into two layers, each with different goals:

### Unit tests (fast, mock environment)

Run via `cargo test --workspace`. These use `soroban_sdk::testutils::Env` — a
mocked, in-process host environment that lets you verify contract logic
quickly without network dependency.

**Coverage:** every implemented entry point — happy paths, error cases,
auth checks, deadline and auto-approval behavior, fee accounting, and
refund/recovery paths. Cross-contract escrow ↔ dispute-resolution flows are
covered by `contracts/campaign-escrow/tests/integration.rs`.

**Limitation:** Unit tests don't exercise real network semantics — actual
transaction submission, real Stellar Asset Contract behavior, ledger time
progression, TTL/rent behavior, or the actual CLI that users will invoke.

### End-to-end testnet smoke test (real network, pre-release verification)

Run via `./scripts/testnet-smoke-test.sh` or trigger via GitHub Actions.

This drives a complete campaign lifecycle against a **real testnet
deployment**:

- Funds testnet accounts via Friendbot
- Orchestrates the full flow: `create_campaign` → `fund_campaign` →
  `apply_to_campaign` → `approve_creator` → `submit_proof` →
  `approve_submission` → `claim_payment`
- Asserts on-chain state at each step, failing with clear errors if
  anything goes wrong
- Exercises the actual `stellar contract invoke` CLI that production users
  will depend on

**When to run:** Before a mainnet deployment (part of the pre-release
checklist), or whenever you want to verify the real deployed artifact works
end-to-end. Not intended to run on every CI push — it has testnet dependency,
funded account setup, and network flakiness.

**How to run:**

```bash
# Against existing testnet deployment (assumes contracts already deployed)
./scripts/testnet-smoke-test.sh

# Deploy fresh contracts first
./scripts/testnet-smoke-test.sh --deploy

# Manually trigger from GitHub Actions
# Go to Actions → Testnet Smoke Test → Run workflow
```

See [`README.md`](../README.md#end-to-end-testnet-smoke-test) for full details.

## Open design questions for contributors

1. **Arbitration model** (`dispute-resolution::resolve_dispute`,
   `campaign-escrow::resolve_dispute_payout`): single trusted arbiter
   (simplest, most centralized) vs. staked juror voting vs. an oracle feed.
   This blocks the last two `todo!()`s. Whatever lands,
   `resolve_dispute_payout` must clear `frozen` / `dispute_opened_at` when it
   settles.
2. **Proof-of-work verification** (`submit_proof`): proofs are currently an
   opaque off-chain URI. An on-chain hash commitment or an oracle attestation
   would make them verifiable.
3. **Version tracking on `upgrade`** (both contracts): `upgrade` swaps the
   WASM but doesn't bump the stored `Version`. Either take a new version
   string or derive it from the wasm hash.

### Settled decisions

These were open questions in the original scaffold and are now decided in
code:

- **Payout sizing:** the business sets `payout_amount` per creator in
  `approve_creator`, and can't commit more than the escrow balance.
- **Fee collection:** `claim_payment` sends the platform fee straight to
  `treasury` on every payout (no accrual or sweep). The fee rate is
  snapshotted into `Campaign.fee_bps` at `create_campaign`, so a later
  `update_fee_bps` (capped at 10%) never changes an existing campaign's
  agreed payouts.
- **Release trigger:** the business accepts a proof with
  `approve_submission` and the creator then calls `claim_payment`. If the
  business never reviews, the creator can claim anyway once
  `completion_deadline` passes (auto-approval). Either side can block this
  by raising a dispute first.

## Known build quirks

- `stellar contract build` prints warnings like `type 'CampaignId' ... is not
  defined in the spec`. This is cosmetic — `CampaignId`/`DisputeId` are
  plain `u64` type aliases (see `contracts/shared/src/lib.rs`) and the
  spec-generation tooling doesn't register a name for bare aliases used
  across a crate boundary. The build still succeeds and the wasm is
  correct.
- `contracts/shared/Cargo.toml` pins `ed25519-dalek = "=2.2.0"` as a
  dev-dependency. This works around `soroban-env-host` declaring an
  unbounded `ed25519-dalek = ">=2.0.0"` dependency, which otherwise resolves
  to a newer semver-major release that breaks `cargo test`'s `testutils`
  build. It's dev-only so it never touches the release/wasm build graph.
  Safe to remove once upstream (`stellar/rs-soroban-env`) tightens that
  bound.
