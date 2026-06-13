#! /bin/bash

pyo3="$(jaq -cr .dependencies.pyo3 Cargo.toml)"
pyo3="${pyo3#0.}"
pyo3="${pyo3%.*}"

echo "pyo3 version: $pyo3"

pyo3_testing="$(jaq -cr .package.version Cargo.toml)"
pyo3_testing="${pyo3_testing#0.}"
pyo3_testing="${pyo3_testing%.*}"

echo "pyo3_testing version: $pyo3_testing"

[[ $pyo3_testing == $pyo3 ]]
