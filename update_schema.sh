#!/usr/bin/env bash

cd contracts

for dir in */; do
    cd $dir
    cargo run --example schema
    cd ..
done

cd ..
