#!/bin/sh

SCRIPT_PATH=$(dirname "$0")

cd "$SCRIPT_PATH/../contracts/campaign" && cargo schema
cd "$SCRIPT_PATH/../contracts/campaign_manager" && cargo schema
cd "$SCRIPT_PATH/../contracts/fund_manager" && cargo schema
cd "$SCRIPT_PATH/../contracts/governance" && cargo schema
cd "$SCRIPT_PATH/../contracts/lp_staking" && cargo schema
cd "$SCRIPT_PATH/../packages/valkyrie_qualifier" && cargo schema
