echo Start CMake libdrivers configuration
echo

cmake -B drivers/build/Release drivers -G Ninja
cp drivers/build/Release/compile_commands.json .
rc=$?

echo

if [ "$rc" -ne 0 ]; then
  echo "CMake libdrivers configuration failed, code=$rc"
else
  echo "CMake libdrivers configuration succeeded !"
fi
echo
