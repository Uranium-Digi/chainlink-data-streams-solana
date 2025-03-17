#!/usr/bin/env bash

set -e

for idl_path_str in "./target/idl"/*
do
  IFS='/' read -r -a idl_path <<< "${idl_path_str}"
  IFS='.' read -r -a idl_name <<< "${idl_path[3]}"
  anchor-go -src "${idl_path_str}" -dst ./client/go/"${idl_name}" -codec borsh
done

cd client/go && go fmt ./...