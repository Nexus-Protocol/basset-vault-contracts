#!/usr/bin/env bash

cd contracts

for dir in */; do
    cd $dir
    cargo schema
    cd ..
done

cd ..
