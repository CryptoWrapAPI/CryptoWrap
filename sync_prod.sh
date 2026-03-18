#!/bin/bash
scp -r scripts/ cw:/cryptowrap/
scp -r app/assets/ cw:/cryptowrap/app/

# restart docker container `main-cw` to apply
scp app/target/release/app cw:/cryptowrap/app/release-cw
# restart nginx to apply
# scp nginx_default.conf cw:/etc/nginx/conf.d/default.conf
# ssl applied, so don't copy

# scp docker-compose.yml cw:/cryptowrap/.
# scp docker-compose.blockchain.yml cw:/cryptowrap/.

# one time only | or copy .env_example and fill the variables
# scp .env cw:/cryptowrap/app/.

# one time only or it will break synced wallet files.
# scp -r crypto/ cw:/cryptowrap/
