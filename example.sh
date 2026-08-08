#!/bin/sh

export ENV_LISTEN_ADDR=127.0.0.1:51804

bin="./target/release/minkvs_bt"

"${bin}"

