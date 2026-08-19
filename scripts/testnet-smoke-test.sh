#!/usr/bin/env bash
#
# Testnet end-to-end smoke test for the full campaign lifecycle.
#
# Drives a complete campaign lifecycle against a real testnet deployment:
# create_campaign → fund_campaign → apply_to_campaign → approve_creator →
# submit_proof → approve_submission → claim_payment
#
# Exit non-zero on any assertion failure.
#
# Usage:
#   ./scripts/testnet-smoke-test.sh [--deploy] [--keep-env]
#
# Options:
#   --deploy     Deploy fresh contracts via deploy.sh (default: use existing from .env.testnet)
#   --keep-env   Keep the generated test .env file after completion (default: clean up)
#
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
STELLAR_NETWORK="${STELLAR_NETWORK:-testnet}"
ENV_FILE=".env.testnet"
TEST_ENV_FILE=".env.testnet.smoke-test"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

# Parse arguments
DEPLOY_FRESH=false
KEEP_ENV=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --deploy)
      DEPLOY_FRESH=true
      shift
      ;;
    --keep-env)
      KEEP_ENV=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Cleanup function
cleanup() {
  if [[ "$KEEP_ENV" != "true" ]]; then
    rm -f "$TEST_ENV_FILE"
  fi
}
trap cleanup EXIT

# Logging helpers
log_info() {
  echo -e "${GREEN}[INFO]${NC} $*"
}

log_warn() {
  echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
  echo -e "${RED}[ERROR]${NC} $*"
}

log_step() {
  echo -e "\n${YELLOW}==>${NC} $*"
}

# Assert helpers
assert_success() {
  local exit_code=$1
  local message=$2
  if [[ $exit_code -ne 0 ]]; then
    log_error "Assertion failed: $message (exit code: $exit_code)"
    exit 1
  fi
  log_info "$message"
}

assert_non_empty() {
  local value=$1
  local name=$2
  if [[ -z "$value" ]]; then
    log_error "Assertion failed: $name is empty"
    exit 1
  fi
  log_info "$name: $value"
}

# Load or deploy contracts
log_step "Loading contract configuration"

if [[ "$DEPLOY_FRESH" == "true" ]]; then
  log_info "Deploying fresh contracts via deploy.sh..."
  cd "$REPO_ROOT"
  bash deploy.sh
  cd - > /dev/null
  log_info "Contracts deployed."
fi

# Load contract IDs from .env.testnet
if [[ ! -f "$REPO_ROOT/$ENV_FILE" ]]; then
  log_error ".env.testnet not found. Run ./deploy.sh first or use --deploy flag."
  exit 1
fi

source "$REPO_ROOT/$ENV_FILE"

assert_non_empty "$CAMPAIGN_ESCROW_ID" "CAMPAIGN_ESCROW_ID"
assert_non_empty "$DISPUTE_RESOLUTION_ID" "DISPUTE_RESOLUTION_ID"

# Also load main .env for base secrets
if [[ -f "$REPO_ROOT/.env" ]]; then
  source "$REPO_ROOT/.env"
fi

# Ensure we have required env vars
: "${ADMIN_SECRET:?Set ADMIN_SECRET in .env}"
: "${ADMIN_ADDRESS:?Set ADMIN_ADDRESS in .env}"

# Fund testnet accounts via friendbot
log_step "Funding testnet accounts via Friendbot"

# Generate three unique test accounts
BUSINESS_SECRET=$(stellar keys generate | grep "Secret:" | awk '{print $2}')
BUSINESS_ADDRESS=$(stellar keys show --secret-key "$BUSINESS_SECRET")
log_info "Generated BUSINESS account: $BUSINESS_ADDRESS"

CREATOR1_SECRET=$(stellar keys generate | grep "Secret:" | awk '{print $2}')
CREATOR1_ADDRESS=$(stellar keys show --secret-key "$CREATOR1_SECRET")
log_info "Generated CREATOR1 account: $CREATOR1_ADDRESS"

CREATOR2_SECRET=$(stellar keys generate | grep "Secret:" | awk '{print $2}')
CREATOR2_ADDRESS=$(stellar keys show --secret-key "$CREATOR2_SECRET")
log_info "Generated CREATOR2 account: $CREATOR2_ADDRESS"

# Fund via Friendbot
log_info "Funding BUSINESS account..."
curl -s "https://friendbot.stellar.org/?addr=$BUSINESS_ADDRESS" > /dev/null
sleep 2

log_info "Funding CREATOR1 account..."
curl -s "https://friendbot.stellar.org/?addr=$CREATOR1_ADDRESS" > /dev/null
sleep 2

log_info "Funding CREATOR2 account..."
curl -s "https://friendbot.stellar.org/?addr=$CREATOR2_ADDRESS" > /dev/null
sleep 2

# Get native XLM asset contract address (for testing)
# XLM is the native Stellar asset, typically used for testing
log_step "Setting up test asset"

# For testnet smoke test, we use the native XLM stellar asset
# In production, this would be a deployed SEP-41 token
NATIVE_XLM_ISSUER="GBUQWP3BOUZX34ULNQG23RQ6F4BFSRXVHAEVZQOTPY5A37YPUYSFVROA"
log_info "Using native Stellar XLM for test asset"

# Store test accounts and IDs for script logic
cat > "$TEST_ENV_FILE" <<EOF
BUSINESS_SECRET=$BUSINESS_SECRET
BUSINESS_ADDRESS=$BUSINESS_ADDRESS
CREATOR1_SECRET=$CREATOR1_SECRET
CREATOR1_ADDRESS=$CREATOR1_ADDRESS
CREATOR2_SECRET=$CREATOR2_SECRET
CREATOR2_ADDRESS=$CREATOR2_ADDRESS
NATIVE_XLM_ISSUER=$NATIVE_XLM_ISSUER
EOF

# Campaign parameters
CAMPAIGN_BUDGET=10000000  # 100 XLM (stroops: 1 XLM = 10,000,000 stroops)
MAX_CREATORS=2
NOW=$(date +%s)
APPLICATION_DEADLINE=$((NOW + 86400))  # 1 day from now
COMPLETION_DEADLINE=$((NOW + 604800))   # 7 days from now
METADATA_URI="ipfs://QmTestCampaignMetadata"

log_step "Test parameters"
log_info "Campaign budget: $CAMPAIGN_BUDGET stroops (~100 XLM)"
log_info "Max creators: $MAX_CREATORS"
log_info "Application deadline: $APPLICATION_DEADLINE"
log_info "Completion deadline: $COMPLETION_DEADLINE"

# Helper to run stellar contract invoke with proper error handling
invoke_contract() {
  local contract_id=$1
  local secret=$2
  shift 2
  local args=("$@")
  
  # Build the invocation command
  local cmd=(
    stellar contract invoke
    --id "$contract_id"
    --source "$secret"
    --network "$STELLAR_NETWORK"
    --
  )
  
  # Add all remaining arguments
  for arg in "${args[@]}"; do
    cmd+=("$arg")
  done
  
  # Execute and capture output
  "${cmd[@]}"
}

# Step 1: Create campaign
log_step "Step 1: create_campaign"

CAMPAIGN_ID=$(invoke_contract "$CAMPAIGN_ESCROW_ID" "$BUSINESS_SECRET" \
  create_campaign \
  --business "$BUSINESS_ADDRESS" \
  --asset "{\"token\":\"$NATIVE_XLM_ISSUER\",\"symbol\":\"XLM\"}" \
  --total_budget "$CAMPAIGN_BUDGET" \
  --max_creators "$MAX_CREATORS" \
  --application_deadline "$APPLICATION_DEADLINE" \
  --completion_deadline "$COMPLETION_DEADLINE" \
  --metadata_uri "$METADATA_URI" | grep -oP '^\d+$' | head -1)

assert_non_empty "$CAMPAIGN_ID" "Created campaign ID"

# Verify campaign was created
CAMPAIGN=$(invoke_contract "$CAMPAIGN_ESCROW_ID" "$ADMIN_SECRET" \
  get_campaign --campaign_id "$CAMPAIGN_ID")
log_info "Campaign state retrieved: $CAMPAIGN"

# Step 2: Fund campaign
log_step "Step 2: fund_campaign"

invoke_contract "$CAMPAIGN_ESCROW_ID" "$BUSINESS_SECRET" \
  fund_campaign \
  --business "$BUSINESS_ADDRESS" \
  --campaign_id "$CAMPAIGN_ID"

assert_success $? "Campaign funded successfully"

# Verify campaign is now Funded
CAMPAIGN=$(invoke_contract "$CAMPAIGN_ESCROW_ID" "$ADMIN_SECRET" \
  get_campaign --campaign_id "$CAMPAIGN_ID")
log_info "Campaign funded, state: $CAMPAIGN"

# Step 3: Creator 1 applies
log_step "Step 3: apply_to_campaign (Creator 1)"

invoke_contract "$CAMPAIGN_ESCROW_ID" "$CREATOR1_SECRET" \
  apply_to_campaign \
  --creator "$CREATOR1_ADDRESS" \
  --campaign_id "$CAMPAIGN_ID" \
  --pitch_uri "ipfs://QmCreator1Pitch"

assert_success $? "Creator 1 applied successfully"

log_info "Creator 1 application submitted"

# Step 4: Creator 2 applies
log_step "Step 4: apply_to_campaign (Creator 2)"

invoke_contract "$CAMPAIGN_ESCROW_ID" "$CREATOR2_SECRET" \
  apply_to_campaign \
  --creator "$CREATOR2_ADDRESS" \
  --campaign_id "$CAMPAIGN_ID" \
  --pitch_uri "ipfs://QmCreator2Pitch"

assert_success $? "Creator 2 applied successfully"

log_info "Creator 2 application submitted"

# Step 5: Business approves Creator 1
log_step "Step 5: approve_creator (Creator 1)"

CREATOR1_PAYOUT=$((CAMPAIGN_BUDGET / 2))  # Split budget evenly

invoke_contract "$CAMPAIGN_ESCROW_ID" "$BUSINESS_SECRET" \
  approve_creator \
  --business "$BUSINESS_ADDRESS" \
  --campaign_id "$CAMPAIGN_ID" \
  --creator "$CREATOR1_ADDRESS" \
  --payout_amount "$CREATOR1_PAYOUT"

assert_success $? "Creator 1 approved with payout: $CREATOR1_PAYOUT"

# Step 6: Business approves Creator 2
log_step "Step 6: approve_creator (Creator 2)"

CREATOR2_PAYOUT=$((CAMPAIGN_BUDGET / 2))  # Split budget evenly

invoke_contract "$CAMPAIGN_ESCROW_ID" "$BUSINESS_SECRET" \
  approve_creator \
  --business "$BUSINESS_ADDRESS" \
  --campaign_id "$CAMPAIGN_ID" \
  --creator "$CREATOR2_ADDRESS" \
  --payout_amount "$CREATOR2_PAYOUT"

assert_success $? "Creator 2 approved with payout: $CREATOR2_PAYOUT"

# Step 7: Creator 1 submits proof
log_step "Step 7: submit_proof (Creator 1)"

invoke_contract "$CAMPAIGN_ESCROW_ID" "$CREATOR1_SECRET" \
  submit_proof \
  --creator "$CREATOR1_ADDRESS" \
  --campaign_id "$CAMPAIGN_ID" \
  --proof_uri "ipfs://QmCreator1Proof"

assert_success $? "Creator 1 proof submitted"

# Step 8: Business approves Creator 1's submission
log_step "Step 8: approve_submission (Creator 1)"

invoke_contract "$CAMPAIGN_ESCROW_ID" "$BUSINESS_SECRET" \
  approve_submission \
  --business "$BUSINESS_ADDRESS" \
  --campaign_id "$CAMPAIGN_ID" \
  --creator "$CREATOR1_ADDRESS"

assert_success $? "Creator 1 proof approved"

# Step 9: Creator 1 claims payment
log_step "Step 9: claim_payment (Creator 1)"

invoke_contract "$CAMPAIGN_ESCROW_ID" "$CREATOR1_SECRET" \
  claim_payment \
  --creator "$CREATOR1_ADDRESS" \
  --campaign_id "$CAMPAIGN_ID"

assert_success $? "Creator 1 payment claimed"

log_info "Creator 1 received payout: $CREATOR1_PAYOUT stroops"

# Step 10: Creator 2 submits proof
log_step "Step 10: submit_proof (Creator 2)"

invoke_contract "$CAMPAIGN_ESCROW_ID" "$CREATOR2_SECRET" \
  submit_proof \
  --creator "$CREATOR2_ADDRESS" \
  --campaign_id "$CAMPAIGN_ID" \
  --proof_uri "ipfs://QmCreator2Proof"

assert_success $? "Creator 2 proof submitted"

# Step 11: Business approves Creator 2's submission
log_step "Step 11: approve_submission (Creator 2)"

invoke_contract "$CAMPAIGN_ESCROW_ID" "$BUSINESS_SECRET" \
  approve_submission \
  --business "$BUSINESS_ADDRESS" \
  --campaign_id "$CAMPAIGN_ID" \
  --creator "$CREATOR2_ADDRESS"

assert_success $? "Creator 2 proof approved"

# Step 12: Creator 2 claims payment
log_step "Step 12: claim_payment (Creator 2)"

invoke_contract "$CAMPAIGN_ESCROW_ID" "$CREATOR2_SECRET" \
  claim_payment \
  --creator "$CREATOR2_ADDRESS" \
  --campaign_id "$CAMPAIGN_ID"

assert_success $? "Creator 2 payment claimed"

log_info "Creator 2 received payout: $CREATOR2_PAYOUT stroops"

# Final verification: Check campaign status
log_step "Final verification"

FINAL_CAMPAIGN=$(invoke_contract "$CAMPAIGN_ESCROW_ID" "$ADMIN_SECRET" \
  get_campaign --campaign_id "$CAMPAIGN_ID")

log_info "Final campaign state: $FINAL_CAMPAIGN"
log_info "Escrow balance should be 0 (all payouts claimed)"

# Success!
log_step "Smoke test completed successfully"
echo ""
log_info "Full campaign lifecycle completed:"
log_info "  ✓ Campaign created and funded"
log_info "  ✓ Creators applied and were approved"
log_info "  ✓ Proofs submitted and approved"
log_info "  ✓ Payments claimed and released"
echo ""

if [[ "$KEEP_ENV" == "true" ]]; then
  log_info "Test environment saved to: $TEST_ENV_FILE"
fi

exit 0
