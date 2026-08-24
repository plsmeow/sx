#!/bin/bash

echo ""

version="$(jq -r '.version' package.json)"
echo "[local] package.json: $version"

version="$(jq -r '.version' package-lock.json)"
echo "[local] package-lock.json: $version"

version="$(sed -n 11p Cargo.toml | cut -c 12-16)"
echo "[local] Cargo.toml: $version"

version="$(jq -r '.version' ./app/tauri.conf.json)"
echo "[local] app/tauri.conf.json: $version"

version="$(jq -r '.version' ./app/tauri.linux.conf.json)"
echo "[local] app/tauri.linux.conf.json: $version"

version="$(jq -r '.version' ./app/tauri.windows.conf.json)"
echo "[local] app/tauri.windows.conf.json: $version"

version="$(sed -n 2p ./app/src/version.rs | cut -c 39-43)"
echo "[local] app/src/version.rs: $version"

version="$(sed -n 2p ./interface/typescript/version.ts | cut -c 32-36)"
echo "[local] interface/typescript/version.ts: $version"

version="$(sed -n 3p ./kernel/src/version.rs | cut -c 39-43)"
echo "[local] kernel/src/version.rs: $version"

version="$(jq -r '.version' salarixi.version.json)"
echo "[public] salarixi.version.json: $version"