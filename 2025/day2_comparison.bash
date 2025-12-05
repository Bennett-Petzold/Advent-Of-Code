#!/usr/bin/env bash

hyperfine -N -w 100 "day2/target/release/day2 day2/input.txt" "day2_opt/target/release/day2_opt day2/input.txt"
