#!/bin/bash

if [[ -z "$1" ]]; then
  echo "Missing test argument"
  echo "Expecting one of:"
  echo "  - step0_repl"
  exit 1
fi

TEST="$1"
if [[ -z "$2" ]]; then
  BIN=$TEST
else
  BIN="$2"
fi


./mal/runtest.py --rundir $(pwd) "./mal/tests/$TEST.mal" ./run.sh $BIN
echo "Ran test suite for $TEST against $BIN binary"