#!/bin/bash

# This script creates a secure tunnel between your production server and local machine
# so you can apply migrations and explore/debug database using something like dbeaver-ce
# don't forget to add database connection host instead of `nakata` in ~/.ssh/config
# preferably using ssh key so you don't need to type password every time
#
# EXAMPLE SSH CONFIGURATION:
# host nakata
  # Hostname 192.168.0.1
  # User nakata
  # Port 22
  # IdentitiesOnly yes
  # IdentityFile ~/id_
#
# you can modify it to work in background if you wish

# ssh -L 5432:127.0.0.1:5432 nakata
ssh -L 5433:127.0.0.1:5432 x10
