add finalizer.py script that would run in background (crontab)

cronjob frequency = 30 minutes

it's purpose is to (passively) check deposits table, update statuses of deposits until they are finalized.
on finalization -> wallet address can be reused again (add some graceful cool-down period, 30 minutes - specific for taking function - e.g. deposit/create)
    (e.g. table monero_wallet, column is_available becomes = true)

but actually address reuse is questionable
