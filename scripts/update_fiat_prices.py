#!/usr/bin/env python3
import os
import sys
import argparse
import psycopg2
from psycopg2.extras import RealDictCursor
from datetime import datetime, timezone
import requests
import dotenv

dotenv.load_dotenv(".env")

COINGECKO_URL = "https://api.coingecko.com/api/v3/simple/price"


def get_db_connection():
    db_url = os.getenv("DB_URL")
    return psycopg2.connect(db_url)


def get_fiat_columns(conn):
    with conn.cursor(cursor_factory=RealDictCursor) as cur:
        cur.execute("""
            SELECT column_name
            FROM information_schema.columns
            WHERE table_name = 'fiat_prices'
              AND column_name NOT IN ('id', 'coin', 'updated_at')
        """)
        return [row["column_name"] for row in cur.fetchall()]


def get_existing_coins(conn):
    with conn.cursor(cursor_factory=RealDictCursor) as cur:
        cur.execute("SELECT coin FROM fiat_prices")
        return [row["coin"] for row in cur.fetchall()]


def add_coin(conn, coin, fiat_currencies):
    print(f"DEBUG: fiat_currencies = {fiat_currencies}")
    
    with conn.cursor(cursor_factory=RealDictCursor) as cur:
        cur.execute("SELECT 1 FROM fiat_prices WHERE coin = %s", (coin,))
        if cur.fetchone():
            print(f"Coin '{coin}' already exists")
            return

        print(f"DEBUG: Fetching prices for coin={coin}, fiats={fiat_currencies}")
        params = {
            "ids": coin,
            "vs_currencies": ",".join(fiat_currencies)
        }
        print(f"DEBUG: API URL params = {params}")
        response = requests.get(COINGECKO_URL, params=params, timeout=30)
        print(f"DEBUG: Response status = {response.status_code}")
        response.raise_for_status()
        prices = response.json()
        print(f"DEBUG: Prices response = {prices}")

        if coin not in prices:
            print(f"Coin '{coin}' not found on CoinGecko")
            return

        values = [prices[coin].get(col) for col in fiat_currencies]
        print(f"DEBUG: values (before adding ts/coin) = {values}")
        values.append(datetime.now(timezone.utc))
        values.append(coin)

        columns = fiat_currencies + ["updated_at", "coin"]
        placeholders = ", ".join(["%s"] * len(columns))
        set_clauses = [f"{col} = EXCLUDED.{col}" for col in fiat_currencies]
        set_clauses.append("updated_at = EXCLUDED.updated_at")

        query = f"""
            INSERT INTO fiat_prices ({", ".join(columns)})
            VALUES ({placeholders})
            ON CONFLICT (coin) DO UPDATE SET {", ".join(set_clauses)}
        """
        print(f"DEBUG: Query = {query}")
        print(f"DEBUG: Values = {values}")
        
        cur.execute(query, values)

        conn.commit()
        print(f"Added/updated coin: {coin} with prices: {prices[coin]}")


def fetch_prices(coins, fiat_currencies):
    if not coins:
        return {}

    coin_ids = ",".join(coins)
    params = {
        "ids": coin_ids,
        "vs_currencies": ",".join(fiat_currencies)
    }

    response = requests.get(COINGECKO_URL, params=params, timeout=30)
    response.raise_for_status()
    return response.json()


def update_prices(conn, prices, fiat_currencies):
    now = datetime.now(timezone.utc)

    set_clauses = [f"{col} = %s" for col in fiat_currencies]
    set_clauses.append("updated_at = %s")
    set_clause = ", ".join(set_clauses)

    with conn.cursor(cursor_factory=RealDictCursor) as cur:
        for coin, fiat_prices in prices.items():
            values = [fiat_prices.get(col) for col in fiat_currencies]
            values.append(now)
            values.append(coin)

            cur.execute(f"""
                UPDATE fiat_prices
                SET {set_clause}
                WHERE coin = %s
            """, values)

    conn.commit()
    print(f"Updated {len(prices)} coins at {now.isoformat()}")


def main():
    parser = argparse.ArgumentParser(description="Update fiat prices from CoinGecko")
    parser.add_argument("--add", metavar="COIN", help="Add a new coin (use CoinGecko ID, e.g., monero)")
    args = parser.parse_args()

    conn = get_db_connection()

    try:
        if args.add:
            print(f"DEBUG: Running add_coin with coin={args.add}")
            fiat_currencies = get_fiat_columns(conn)
            print(f"DEBUG: Got fiat_currencies = {fiat_currencies}")
            if not fiat_currencies:
                print("No fiat currency columns found")
                return
            add_coin(conn, args.add, fiat_currencies)
            return

        fiat_currencies = get_fiat_columns(conn)
        print(f"Found fiat columns: {fiat_currencies}")

        if not fiat_currencies:
            print("No fiat currency columns found")
            return

        coins = get_existing_coins(conn)
        print(f"Found {len(coins)} coins in database: {coins}")

        if not coins:
            print("No coins to update")
            return

        prices = fetch_prices(coins, fiat_currencies)
        print(f"Fetched prices for: {list(prices.keys())}")

        update_prices(conn, prices, fiat_currencies)

    except requests.RequestException as e:
        print(f"Error fetching prices from CoinGecko: {e}")
    except Exception as e:
        print(f"Error: {e}")
    finally:
        conn.close()


if __name__ == "__main__":
    main()


'''

Usage:
  # Add a new coin (use CoinGecko ID, e.g., monero for XMR)
  python3 scripts/update_fiat_prices.py --add monero

  # Update all prices
  python3 scripts/update_fiat_prices.py

  # For crontab (e.g., every 5 minutes):
  */5 * * * * /usr/bin/python3 /home/nmodem/Desktop/project/cryptowrap.cv/scripts/update_fiat_prices.py >> /var/log/fiat_prices.log 2>&1

Requirements:
  pip install psycopg2-binary python-dotenv requests

Note: Use CoinGecko IDs (e.g., monero, bitcoin, ethereum) not exchange symbols (XMR, BTC, ETH)

'''
