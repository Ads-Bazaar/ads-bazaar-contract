# AdsBazaar Contracts

**Soroban smart contracts for AdsBazaar — a decentralized marketplace for multi-currency creator campaigns on Stellar.**

AdsBazaar helps businesses fund influencer marketing campaigns in the currency they already use, while creators receive escrow-protected payouts through Stellar assets and local payment rails.

The initial focus is emerging-market creator commerce: Nigerian businesses paying in Naira-denominated assets, Kenyan creators withdrawing through mobile-money-connected anchors, and global teams settling campaigns in stablecoins without rebuilding the same trust and FX workflow for every country.

This repository is the on-chain layer of the [AdsBazaar](https://twitter.com/AdsBazaar5) product: the Soroban contracts that hold campaign budgets in escrow and arbitrate contested payouts. The frontend and backend live in a separate repository.

> [!NOTE]
> The full campaign escrow lifecycle is implemented and tested: campaign creation, funding, creator applications and approval, proof submission and review, payout claiming with platform fee, cancellation, expiry, surplus reclaim, and raising and freezing disputes. One piece is still open: settling a dispute through the arbiter in `dispute-resolution` (`resolve_dispute` → `campaign-escrow::resolve_dispute_payout`), which is still `todo!()` because the arbitration model hasn't been decided yet. Until then, an admin can settle disputes through `campaign-escrow::resolve_dispute`, which only works after a 72-hour evidence window. The contracts haven't been audited yet and aren't on mainnet.

---

## Why AdsBazaar

Influencer marketing works poorly outside USD-first markets.

Businesses in Africa, Latin America, and other high-growth regions often pay creators through manual transfers, informal agreements, or centralized platforms that require USD conversion. The result is predictable: high fees, delayed payouts, limited recourse for creators, and operational overhead for brands trying to run cross-border campaigns.

AdsBazaar is designed around a simpler primitive:

1. A business funds a campaign in a Stellar-supported asset.
2. Funds move into a Soroban escrow contract.
3. Creators apply, are selected, submit proof of work, and claim payment when approved.
4. Local deposit and withdrawal providers handle fiat entry and exit through standardized Stellar anchor flows.

The goal is not to add crypto complexity to marketing. The goal is to remove the payment and trust overhead that prevents small businesses and independent creators from working across borders.

---

## Why Stellar

AdsBazaar is built for multi-currency payments first. Stellar is a strong fit because its core network, ecosystem standards, and Soroban contracts are designed around asset movement rather than speculative execution.

- **Native multi-asset infrastructure** — Stellar accounts can hold XLM, USDC, EURC, and anchored local-currency assets as first-class network assets. A campaign can be denominated in whatever asset makes sense for the business, without wrapping every payment path in a new token contract.
- **Low-fee settlement** — creator campaigns often involve many small payouts; Stellar's low transaction costs make paying ten or a hundred creators practical.
- **Soroban smart contracts** — deterministic, Rust-based escrow logic for campaign funding, creator selection, proof submission, approval, and payout claiming. This repo's contract layer is intentionally small: custody and state transitions are enforced on-chain, while discovery, notifications, identity, and anchor sessions live off-chain (see `apps/backend` in the frontend repo).
- **SEP-24 anchors** — a standardized interactive deposit/withdrawal flow. This is what lets a business or creator enter and exit through familiar local rails (bank transfer, mobile money) instead of a crypto-native onramp. Anchor orchestration lives in the backend service, not in these contracts — the contracts only ever see a Stellar asset that's already been deposited.

---

## Contract Architecture

```
contracts/
├── shared/               ads-bazaar-shared          — common types, no #[contract] of its own
├── campaign-escrow/      ads-bazaar-campaign-escrow  — holds & releases campaign funds
└── dispute-resolution/   ads-bazaar-dispute-resolution — arbitrates contested payouts
docs/
└── ARCHITECTURE.md       design overview + open questions for contributors
```

Two contracts instead of one, on purpose:

- **Escrow correctness is the highest-stakes code in this repo** — it holds real business funds, so its surface area (and audit surface) is kept as small as possible.
- **Arbitration is the least-settled design space.** Whether disputes are resolved by a single trusted arbiter, a staked jury, or an oracle is an open question. Isolating it in its own contract means that design can iterate — or even be redeployed — without touching escrow.

The two contracts talk to each other through a narrow, explicit interface, not a shared database. `freeze_for_dispute` and `resolve_dispute_payout` on escrow can only be called by the configured dispute contract (or, for freezing, the admin). `get_campaign_business` lets the dispute contract check who owns a campaign. `close_dispute` on the dispute contract can only be called by escrow.

### Campaign lifecycle

```rust
pub enum CampaignStatus {
    Draft,      // created but not yet funded
    Funded,     // escrow balance deposited, open for applications
    Active,     // at least one creator approved and producing content
    Completed,  // all approved creators paid out
    Cancelled,  // refunded to the business before completion
    Disputed,   // flagged for dispute; funds locked pending resolution
}
```

The happy path for one creator is `create_campaign` → `fund_campaign` → `apply_to_campaign` → `approve_creator` → `submit_proof` → `approve_submission` → `claim_payment`. If the business doesn't review a submitted proof by the `completion_deadline`, the creator can claim anyway (auto-approval).

### Contract capabilities

`campaign-escrow`:

| Function | Purpose | Status |
| --- | --- | --- |
| `initialize` | Sets admin, trusted dispute contract, platform fee bps | Implemented |
| `create_campaign` | Business creates a draft campaign for a given `PayoutAsset` (checks that the token is a live SEP-41 contract, and checks budget, creator cap, and deadline order) | Implemented |
| `fund_campaign` | Transfers the full campaign budget from the business into escrow (`Draft` → `Funded`) | Implemented |
| `update_campaign_metadata` | Business amends the campaign brief before any creator has applied | Implemented |
| `apply_to_campaign` | Creator applies once to a funded campaign before the application deadline | Implemented |
| `approve_creator` | Business approves an applicant and sets their payout. Rejects going over `max_creators`, approving the same creator twice, committing more than the escrow holds, or approving after the deadline | Implemented |
| `submit_proof` | Approved creator submits proof of completed work before the completion deadline | Implemented |
| `approve_submission` / `reject_submission` | Business accepts a proof (makes the payout claimable) or rejects it (creator can resubmit) | Implemented |
| `claim_payment` | Creator claims their escrowed payout minus the platform fee, which goes to treasury. Works after approval, or after the deadline via auto-approval | Implemented |
| `cancel_campaign` / `expire_campaign` / `reclaim_surplus` | Business gets back the escrow that isn't committed to any creator. Payouts already committed stay claimable | Implemented |
| `emergency_recover_campaign` | Admin sweeps uncommitted funds from a campaign abandoned ~6 months past its deadline, sending them to treasury | Implemented |
| `freeze_for_dispute` | Freezes one creator's payout while a dispute is reviewed and starts the 72h evidence window | Implemented |
| `resolve_dispute` | Admin settles a frozen payout after `MIN_EVIDENCE_WINDOW`, then closes the dispute record | Implemented |
| `resolve_dispute_payout` | Arbiter-driven settlement hook called by `dispute-resolution` | `todo!()` |
| `pause` / `unpause`, `propose_admin` / `accept_admin`, `update_fee_bps` / `update_treasury`, `upgrade` | Admin controls: pause switch, two-step admin transfer, fee capped at 10%, treasury address, WASM upgrade | Implemented |
| `applicant_count` / `campaign_applicants` | Count of a campaign's applicants, and a page of their addresses in the order they applied (up to `MAX_APPLICANTS_PAGE` = 50 per call) | Implemented |
| `get_campaign` / `get_application` / `get_campaign_business` / `get_protocol_config` / `is_paused` / `version` | Read-only lookups | Implemented |

`dispute-resolution`:

| Function | Purpose | Status |
| --- | --- | --- |
| `initialize` | Sets admin and the trusted escrow contract address | Implemented |
| `raise_dispute` | Creator or campaign business raises a dispute over one payout and freezes it in escrow (one open dispute per campaign/creator pair) | Implemented |
| `assign_arbiter` | Admin assigns an arbiter to a raised dispute (`Raised` → `UnderReview`) | Implemented |
| `resolve_dispute` | Arbiter resolves a dispute and triggers payout via escrow | `todo!()` |
| `close_dispute` | Escrow-only callback that marks a dispute resolved after escrow's admin `resolve_dispute` settles it | Implemented |
| `get_dispute` / `version` / `upgrade` | Read-only lookups and admin WASM upgrade | Implemented |

Each of the two remaining `todo!()`s has a doc comment above it in `lib.rs`. Both are blocked on the same open question: which arbitration model to use (single trusted arbiter, staked jurors, or an oracle).

### Escrow design principles

- No embedded exchange routing — the contract escrows and releases exactly the asset the business funded it with, via the standard `soroban_sdk::token::Client` (SEP-41). Currency conversion, if any, happens off-chain before funding.
- No oracle dependency for payout execution.
- No custody by the application backend — only the contract holds funds.
- No hidden admin release path — release/refund/dispute-resolution hooks are the only ways funds move.
- Structured `#[contractevent]` events for every state transition, for indexers and campaign activity feeds.

---

## Multi-currency asset model

A campaign is funded in whatever Stellar asset the business already holds, represented by:

```rust
pub struct PayoutAsset {
    pub token: Address,   // any SEP-41-compatible token contract
    pub symbol: String,   // display-only, e.g. "USDC", "NGNC" — not trusted on-chain
}
```

`token` can be a classic Stellar Asset Contract (XLM, a Naira-pegged stablecoin, USDC, EURC) or a native Soroban token. The escrow contract is asset-agnostic by design — this is what lets a Lagos business fund a campaign in a Naira-denominated asset and a Nairobi creator withdraw through a mobile-money-connected anchor without either side touching a different contract.

| Asset | Region | Expected rail |
| --- | --- | --- |
| XLM | Global | Native Stellar account funding and fees |
| USDC | Global | Circle-issued Stellar USDC |
| EURC | Europe | Euro stablecoin |
| NGN-denominated asset | Nigeria | Bank-transfer-connected Stellar anchor |
| KES-denominated asset | Kenya | Mobile-money-connected anchor |

> [!IMPORTANT]
> Asset symbols, issuers, and trustline requirements should be configured from verified network metadata before production deployment. `PayoutAsset.symbol` is display-only and must never be trusted for on-chain logic.

---

## Repository structure

```
.
├── contracts
│   ├── shared/               common types (CampaignStatus, PayoutAsset, ...)
│   ├── campaign-escrow/      escrow contract
│   └── dispute-resolution/   arbitration contract
├── docs
│   └── ARCHITECTURE.md
├── scripts
│   └── testnet-smoke-test.sh  end-to-end lifecycle check against testnet
├── .github/workflows
│   ├── ci.yml                 build, lint, test
│   └── testnet-smoke-test.yml manually triggered smoke test
├── deploy.sh                  build + deploy + cross-initialize both contracts
├── Cargo.toml                 workspace manifest
└── rust-toolchain.toml
```

---

## Local development

### Prerequisites

- [Rust](https://rustup.rs/) — toolchain and wasm targets (`wasm32-unknown-unknown`, `wasm32v1-none`) are pinned in `rust-toolchain.toml` and installed automatically by `rustup`
- [Stellar CLI](https://developers.stellar.org/docs/tools/cli/install-cli) (`stellar`) `>= 22.0.0` for building, optimizing, deploying, and invoking contracts

### Build & test

```bash
# run all unit tests
cargo test --workspace

# lint + format
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

# build deployable .wasm for every contract
stellar contract build
```

### Deploy to testnet

1. Fund a Stellar testnet account (for the deployer and, if you want a
   separate admin, a second one) via
   [friendbot](https://developers.stellar.org/docs/tools/quickstart#fund-your-account),
   and generate keys with `stellar keys generate`.
2. Copy the env template and fill in your keys:

   ```bash
   cp .env.example .env
   ```

3. Run the deploy script:

   ```bash
   ./deploy.sh
   ```

   This builds and optimizes both contract wasms, deploys
   `campaign-escrow` and `dispute-resolution`, initializes each with the
   other's address (so the dispute hooks can authenticate each other),
   and appends the resulting contract IDs to `.env.testnet`.

#### Manual invocation examples

Once deployed (`$ESCROW_ID` / `$DISPUTE_ID` from `.env.testnet`, `$BUSINESS_SECRET` / `$CREATOR_SECRET` / `$TOKEN_ID` from your own testnet setup):

```bash
# Business creates a draft campaign
stellar contract invoke --id "$ESCROW_ID" --source "$BUSINESS_SECRET" --network testnet \
  -- create_campaign \
  --business "$BUSINESS_ADDRESS" \
  --asset '{"token":"'"$TOKEN_ID"'","symbol":"USDC"}' \
  --total_budget 1000000 \
  --max_creators 5 \
  --application_deadline 1750000000 \
  --completion_deadline 1752000000 \
  --metadata_uri "ipfs://brief"

# Business funds the campaign (transfers total_budget into escrow)
stellar contract invoke --id "$ESCROW_ID" --source "$BUSINESS_SECRET" --network testnet \
  -- fund_campaign --business "$BUSINESS_ADDRESS" --campaign_id 0

# Creator applies
stellar contract invoke --id "$ESCROW_ID" --source "$CREATOR_SECRET" --network testnet \
  -- apply_to_campaign --creator "$CREATOR_ADDRESS" --campaign_id 0 --pitch_uri "ipfs://pitch"

# Business approves the creator and sets their payout
stellar contract invoke --id "$ESCROW_ID" --source "$BUSINESS_SECRET" --network testnet \
  -- approve_creator --business "$BUSINESS_ADDRESS" --campaign_id 0 \
  --creator "$CREATOR_ADDRESS" --payout_amount 200000

# Creator submits proof of completed work
stellar contract invoke --id "$ESCROW_ID" --source "$CREATOR_SECRET" --network testnet \
  -- submit_proof --creator "$CREATOR_ADDRESS" --campaign_id 0 --proof_uri "ipfs://proof"

# Business approves the submitted proof
stellar contract invoke --id "$ESCROW_ID" --source "$BUSINESS_SECRET" --network testnet \
  -- approve_submission --business "$BUSINESS_ADDRESS" --campaign_id 0 --creator "$CREATOR_ADDRESS"

# Creator claims payment (receives payout minus platform fee)
stellar contract invoke --id "$ESCROW_ID" --source "$CREATOR_SECRET" --network testnet \
  -- claim_payment --creator "$CREATOR_ADDRESS" --campaign_id 0

# Read-only lookup
stellar contract invoke --id "$ESCROW_ID" --source "$BUSINESS_SECRET" --network testnet \
  -- get_campaign --campaign_id 0
```

#### Contract addresses

| Network | campaign-escrow | dispute-resolution |
| --- | --- | --- |
| Testnet | _fill in after first `./deploy.sh` run_ | _fill in after first `./deploy.sh` run_ |
| Mainnet | not yet deployed | not yet deployed |

### End-to-end testnet smoke test

Before a mainnet deployment, verify the full campaign lifecycle against a real testnet deployment:

```bash
# Run the smoke test against an existing testnet deployment
./scripts/testnet-smoke-test.sh

# Or deploy fresh contracts and run the smoke test
./scripts/testnet-smoke-test.sh --deploy

# Keep the test environment file for inspection
./scripts/testnet-smoke-test.sh --keep-env
```

The smoke test:
- Funds three testnet accounts via [Friendbot](https://developers.stellar.org/docs/tools/quickstart#fund-your-account)
- Drives a complete campaign lifecycle: `create_campaign` → `fund_campaign` → `apply_to_campaign` (2 creators) → `approve_creator` (both) → `submit_proof` (both) → `approve_submission` (both) → `claim_payment` (both)
- Asserts on-chain state at each step, failing loudly with clear errors if anything goes wrong
- Exercises real network semantics: actual transaction submission, Stellar asset behavior, ledger time progression, and the actual `stellar contract invoke` CLI that users will rely on

This is the verification step that exercises the real deployed artifact, not just the unit tests in `cargo test`.

#### Manual testnet smoke test trigger

Manually trigger the smoke test workflow from GitHub Actions:

1. Go to **Actions** → **Testnet Smoke Test**
2. Click **Run workflow**
3. Optionally check **Deploy fresh contracts** to deploy new contracts before testing
4. Wait for completion and check logs

This is the recommended pre-release checklist item to confirm the testnet deployment works end-to-end.

---

## Testing

`cargo test --workspace` runs 154 tests: unit tests in each contract's `src/test.rs` and cross-contract integration tests in `contracts/campaign-escrow/tests/integration.rs`. The testnet smoke test above checks the same lifecycle against a real deployment.

| Area | Covered |
| --- | --- |
| Escrow funding/release | Funding, payout claims, fee accounting (including fee snapshot and updates), duplicate claims, deadline and auto-approval behavior |
| Campaign workflow | Application limits, selection permissions, proof submission/rejection/resubmission, metadata updates |
| Refunds & recovery | Cancel, expire, surplus reclaim with committed payouts still outstanding, emergency recovery grace period |
| Disputes | Who can raise a dispute, freezing a single payout, status transitions, the evidence window, admin settlement and dispute-record sync |
| Admin | Pause guard, two-step admin transfer, fee cap, treasury updates |
| Cross-contract | `freeze_for_dispute` / `close_dispute` caller restrictions, escrow ↔ dispute-resolution round trips |

When `resolve_dispute` / `resolve_dispute_payout` are implemented, add tests for applying arbiter outcomes and for which callers each hook accepts (see `CONTRIBUTING.md`).

---

## Security considerations

These contracts handle payment workflows and should be treated as financial infrastructure.

- Minimize contract surface area and keep escrow logic auditable.
- Require wallet authorization (`require_auth`) for every business/creator action.
- No backend custody of campaign funds — only the contract holds them.
- Emit contract events for independent indexing and reconciliation.
- Keep issuer addresses, anchor metadata, and network configuration explicit — never hardcoded assumptions in contract logic.
- Add contract tests for all payout and dispute edge cases before mainnet deployment.
- Complete external review before handling production campaign value.

Known areas requiring design work before production: the arbiter-driven dispute resolution model, fee governance policy beyond the current admin-set fee (capped at 10%), version tracking across `upgrade`, and an independent contract audit.

---

## Roadmap

| Phase | Focus | Status |
| --- | --- | --- |
| 0 | Workspace scaffold: data model, storage, errors, events, CI | Done |
| 1 | Campaign creation, funding, and escrow release logic | Done |
| 2 | Creator application, approval, and proof submission flow | Done |
| 3 | Dispute arbitration model and cross-contract dispute hooks | In progress: raise/freeze/assign and admin settlement done; arbiter resolution open |
| 4 | Testnet deployment and integration with the backend indexer | In progress: deploy script and smoke test in place; indexer integration pending |
| 5 | External audit and mainnet launch | Planned |

---

## Contributing

Contributions are welcome, especially in areas where Stellar infrastructure, emerging-market payments, and creator marketplace design intersect.

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the full guide.

### Open contribution areas

- Arbitration model proposal, then `dispute-resolution::resolve_dispute` and `campaign-escrow::resolve_dispute_payout` (must clear `frozen` / `dispute_opened_at` when it settles)
- Version tracking for `upgrade` (see the TODO on `upgrade` in both contracts)
- Proof-of-work verification design for `submit_proof` (proofs are currently opaque URIs)
- Backend indexer integration against the emitted `#[contractevent]` events

Open an issue before starting large protocol or state-machine changes.

---

## Current status

Under active development. Not yet audited or on mainnet.

- Campaign escrow lifecycle (create, fund, apply, approve, prove, review, claim, cancel/expire/reclaim, emergency recovery): implemented and tested.
- Admin controls (pause, two-step admin transfer, fee and treasury updates, upgrade): implemented and tested.
- Disputes: raising, freezing, arbiter assignment, and admin settlement after the evidence window are implemented. Arbiter-driven resolution is still `todo!()`.
- Testnet: `deploy.sh` and an end-to-end smoke test workflow are in place. Published contract addresses are pending.
- External audit: not yet started.

---

## License

[MIT](LICENSE)
