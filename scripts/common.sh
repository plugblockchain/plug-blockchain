#!/usr/bin/env bash

ROOT=`dirname "$0"`

# A list of directories which contain wasm projects.
SRCS=(
	"core/executor/wasm"
	"node/runtime/wasm"
	# "node-template/runtime/wasm" # TODO: fix node-template issue (#9)
	"core/test-runtime/wasm"
)

# Make pushd/popd silent.

pushd () {
	command pushd "$@" > /dev/null
}

popd () {
	command popd "$@" > /dev/null
}
