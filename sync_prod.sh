#!/bin/bash
# scp -r scripts/ spb:/cryptowrap/
scp -r app/assets/ spb:/cryptowrap/app/

# restart docker container `main-cw` to apply
scp app/target/release/app spb:/cryptowrap/app/release-cw
# restart nginx to apply
# scp nginx/nginx_default.conf spb:/etc/nginx/conf.d/default.conf
# after first copy, issue a tls sertificate using certbot

# scp docker-compose.yml spb:/cryptowrap/.
#
# monero container args changed, so don't copy over again
# scp docker-compose.blockchain.yml spb:/cryptowrap/.

# one time only | or copy .env_example and fill the variables
# scp .env spb:/cryptowrap/app/.

# one time only or it will break synced wallet files.
# scp -r crypto/ spb:/cryptowrap/


# TODO: later orginize project so it can be deployed with a single bash script
# and some way to update binary in-place and restart docker container easily
