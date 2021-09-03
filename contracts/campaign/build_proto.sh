#!/bin/sh

protoc --rust_out "./src/proto" --proto_path "./proto" "core_response.proto"
