set confirm off
set pagination off

# Se connecte au serveur GDB d’OpenOCD
target extended-remote :3333

# Optionnel mais utile pour traces semihosting (hprintln!, etc.)
monitor arm semihosting enable

# Reset + halt avant chargement
monitor reset halt

# Charge l’ELF envoyé par Cargo (GDB est lancé avec le fichier en argument)
load

# Se positionne au début (ou relance) et s’arrête prêt à debugger
monitor init
monitor arm semihosting enable
monitor reset halt

# Supprime tous les breakpoints
delete breakpoints

# Placez des breakpoints si besoin, puis:
continue
