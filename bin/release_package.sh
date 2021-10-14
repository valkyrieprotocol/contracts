#!/bin/sh

PROJECT_PATH="$(dirname "$0")/.."

cd "$PROJECT_PATH/packages/valkyrie" && cargo publish
cd "$PROJECT_PATH/packages/valkyrie_qualifier" && cargo publish