#!/bin/sh

PROJECT_PATH="$(dirname "$0")/.."

run () {
  cd "$PROJECT_PATH/$1" && cargo schema
  rm "$PROJECT_PATH/target/debug/examples/schema"
}

run "contracts/campaign"
run "contracts/community"
run "contracts/distributor"
run "contracts/governance"
run "contracts/lp_staking"
run "packages/valkyrie_qualifier"
