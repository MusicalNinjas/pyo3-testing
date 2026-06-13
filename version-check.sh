#! /bin/bash

pyo3="$(jaq -cr .dependencies.pyo3 Cargo.toml)"
pyo3="${pyo3#0.}" # strip leading `0.` if present
pyo3="${pyo3%%.*}" # strip everything after the next `.`

echo "pyo3 version: $pyo3"

pyo3_testing="$(jaq -cr .package.version Cargo.toml)"
pyo3_testing="${pyo3_testing#0.}"
pyo3_testing="${pyo3_testing%%.*}"

echo "pyo3_testing version: $pyo3_testing"

[[ $pyo3_testing == $pyo3 ]]
