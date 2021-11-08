#!/bin/sh

PROJECT_PATH="$(dirname "$0")/.."

run () {
  rm "$PROJECT_PATH/target/debug/examples/schema"
  cd "$PROJECT_PATH/$1" && cargo schema
}

run "contracts/campaign"
run "contracts/campaign_manager"
run "contracts/community"
run "contracts/distributor"
run "contracts/governance"
run "contracts/lp_staking"
run "packages/valkyrie_qualifier"
