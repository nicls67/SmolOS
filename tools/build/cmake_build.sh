echo Start libdrivers build
echo

cmake --build drivers/build/Release
rc=$?

echo

if [ "$rc" -ne 0 ]; then
  echo "libdrivers build failed, code=$rc"
else
  echo "libdrivers build succeeded !"
fi
echo
