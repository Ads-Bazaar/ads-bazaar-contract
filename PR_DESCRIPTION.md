# Implement escrow lifecycle & comprehensive test suite

## Summary

This PR delivers the prerequisite test coverage requested in **issue #19** and,
because the escrow state-transition logic was still a set of `todo!()` stubs, it
also implements that logic so every scenario can actually run and pass.

The campaign-escrow contract now implements the full creator-payout lifecycle
against a SEP-41 token (`soroban_sdk::token::Client`), with platform fees,
cancellation, expiry and surplus reclaim.

## What changed

### `contracts/campaign-escrow/src/lib.rs`
Implemented the following previously-stubbed entry points:

- `create_campaign` – validates budget/creators/deadlines; mints a `Draft` `Campaign`.
- `fund_campaign` – pulls `total_budget` from the business into escrow (`Draft → Funded`).
- `apply_to_campaign` – creator applies before the application deadline (once only).
- `approve_creator` ("select") – business selects a creator and sets their payout,
  guarding `max_creators` and over-committing escrow.
- `submit_proof` – approved creator submits proof before the content deadline.
- `approve_submission` / `reject_submission` – business accepts or bounces a submission
  (rejection lets the creator re-submit).
- `claim_payment` – releases the escrowed payout minus the configured platform fee;
  auto-approves once the content deadline has passed.
- `cancel_campaign` – refunds remaining escrow to the business.
- `expire_campaign` – refunds full remaining escrow after the content deadline.
- `reclaim_surplus` – returns unallocated escrow after payouts are released.

### `contracts/campaign-escrow/src/error.rs`
Added the error variants required by the spec:
`NotCampaignOwner`, `SubmissionNotPayable`, `AlreadyApplied`, `AlreadySelected`,
`ApplicationDeadlinePassed`, `ContentDeadlinePassed`, `DeadlineInPast`,
`DeadlineNotReached`.

### `contracts/campaign-escrow/src/types.rs`
Added `committed_payouts` to `Campaign` (reserved against `escrow_balance`) and
`proof_approved` to `Application`.

### `contracts/campaign-escrow/src/test.rs`
Extracted a `test_helpers` module (`setup_env`, `setup_token`, `advance_time`,
plus `bootstrap` / `create_funded_campaign` / `usdc` helpers) and added three
test modules covering every scenario from issue #19:

- **`test_happy_path`**: `full_lifecycle`, `fee_calculation_50bps`,
  `auto_approve_past_deadline`, `cancel_open_campaign`,
  `reclaim_surplus_after_payouts`, `reject_and_resubmit`, `expire_no_submissions`.
- **`test_auth_failures`**: `non_owner_cancel`, `non_owner_select_creator`,
  `non_owner_approve_submission`, `creator_claim_before_approval`,
  `double_apply_same_creator`, `double_select_same_creator`.
- **`test_deadline_enforcement`**: `apply_after_application_deadline`,
  `submit_after_content_deadline`, `create_with_past_deadline`,
  `expire_before_deadline`.

Balance assertions use `token::Client::new(&env, &asset).balance(&address)` and
ledger time is advanced via `env.ledger().with_mut(|l| l.timestamp = ...)`.
All tests run under `env.mock_all_auths()`.

## Acceptance criteria

- All listed test scenarios implemented and passing (21 tests, 0 failures).
- `cargo test` exits 0 with zero warnings.
- `cargo clippy` is clean for the crate.

## Test plan

```
cargo test
```

closes #19
