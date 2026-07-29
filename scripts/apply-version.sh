#!/usr/bin/env bash

VERSION=$(cat VERSION)

sed -i.bak "13s/version = \".*\"/version = \"$VERSION\"/g" Cargo.toml

rm Cargo.toml.bak
