#!/usr/bin/env bash

find target/release/ -maxdepth 1 -type f ! -name '.*' ! -name '*.d' |
	awk '{ print $0 " input 75"}' |
	tr '\n' '\0' |
	xargs -0 hyperfine -N -w 1000 --export-markdown release-bench.md

# If you also want full perf runs
#find target/full-perf/ -maxdepth 1 -type f ! -name '.*' ! -name '*.d' |
#	awk '{ print $0 " input 75"}' |
#	tr '\n' '\0' |
#	xargs -0 hyperfine -N -w 1000 --export-markdown full-perf-bench.md
