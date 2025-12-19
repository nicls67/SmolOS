echo Start generating drivers allocation
echo

python3 tools/gen_drivers_alloc/gen_drivers_alloc.py  config/drivers_conf.yaml
rc=$?

echo

if [ "$rc" -ne 0 ]; then
  echo "Generation of driver allocation failed, code=$rc"
else
  echo "Generation of driver allocation succeeded !"
fi
echo
