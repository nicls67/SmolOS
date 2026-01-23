#!/usr/bin/env bash

l_archive_path="${1:-drivers/drivers.tar.gz}"

if [ ! -d drivers/Core ] || [ ! -d drivers/Drivers ]; then
  echo "Missing drivers/Core or drivers/Drivers. Run from repo root."
  exit 1
fi

echo "Creating archive: $l_archive_path"
tar -czf "$l_archive_path" drivers/Core drivers/Drivers
rc=$?

echo
if [ "$rc" -ne 0 ]; then
  echo "Archive creation failed, code=$rc"
else
  echo "Archive created successfully."
fi
