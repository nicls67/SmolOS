#!/usr/bin/env bash

l_archive_path="${1:-drivers/drivers.tar.gz}"

if [ ! -f "$l_archive_path" ]; then
  echo "Archive not found: $l_archive_path"
  exit 1
fi

if [ ! -d drivers ]; then
  echo "Missing drivers/ directory. Run from repo root."
  exit 1
fi

echo "Extracting archive: $l_archive_path"
tar -xzf "$l_archive_path" -C .
rc=$?

echo
if [ "$rc" -ne 0 ]; then
  echo "Archive extraction failed, code=$rc"
else
  echo "Archive extracted successfully."
fi
