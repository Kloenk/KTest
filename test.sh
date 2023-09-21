#!/usr/bin/env bash

. "$KTEST_TEST_LIB"

config-arch $(uname -m)

require-kernel-config FOO=m

main "$@"