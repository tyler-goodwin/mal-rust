#!/bin/bash

if [[ -z "$1" ]]; then
  echo "Missing test argument"
  echo "Expecting one of:"
  echo "  - step0_repl"
  exit 1
fi

./mal/runtest.py --rundir $(pwd) "./mal/tests/$1.mal" cargo run $1