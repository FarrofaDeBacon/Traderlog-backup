import random
import uuid
from datetime import datetime, timedelta
import json
import os

# --- Stress Test Seeding Tool ---
# Goal: Generate 10,000 realistic trades in Svelte 5 / SurrealDB
# Outputs a .surql file that can be imported or a JSON for batch processing.

COUNT = 10000
ACCOUNTS = ["account:⟨demo-real⟩", "account:⟨demo-sim⟩"]
STRATEGIES = ["strategy:⟨strat-price-action⟩", "strategy:⟨strat-trend-follower⟩"]
ASSETS = ["WINJ24", "WDOM24", "PETR4", "VALE3", "AAPL"]
TYPES = ["asset_type:⟨future⟩", "asset_type:⟨stock⟩"]

def generate_trades():
    trades = []
    base_date = datetime.now() - timedelta(days=365)
    
    print(f"Generating {COUNT} stress test trades...")
    
    with open("stress_data.surql", "w", encoding="utf-8") as f:
        # Optimization: Use a single transaction for SurrealDB
        f.write("BEGIN TRANSACTION;\n")
        
        for i in range(COUNT):
            trade_id = str(uuid.uuid4())
            date = base_date + timedelta(minutes=random.randint(1, 525600))
            exit_date = date + timedelta(minutes=random.randint(1, 480))
            
            asset = random.choice(ASSETS)
            qty = random.randint(1, 100)
            entry = random.uniform(50000, 130000) if "WIN" in asset else random.uniform(20, 150)
            exit_p = entry + random.uniform(-500, 1000) if "WIN" in asset else entry + random.uniform(-2, 5)
            
            result = (exit_p - entry) * qty
            if "WIN" in asset:
                result *= 0.2 # ticks to points
            
            is_win = result > 0
            
            # SurrealQL Format
            query = f"UPSERT trade:⟨{trade_id}⟩ CONTENT {{ "
            query += f"account_id: type::thing('{random.choice(ACCOUNTS)}'), "
            query += f"asset_symbol: '{asset}', "
            query += f"asset_type_id: type::thing('{random.choice(TYPES)}'), "
            query += f"date: '{date.isoformat()}Z', "
            query += f"entry_price: {entry}, "
            query += f"exit_date: '{exit_date.isoformat()}Z', "
            query += f"exit_price: {exit_p}, "
            query += f"followed_plan: {random.choice([True, True, False])}, "
            query += f"quantity: {qty}, "
            query += f"result: {result}, "
            query += f"status: 'Closed', "
            query += f"strategy_id: type::thing('{random.choice(STRATEGIES)}'), "
            query += f"notes: 'Stress test generated trade #{i}' "
            query += "};\n"
            
            f.write(query)
            
            if i % 1000 == 0:
                print(f"  Progress: {i}/{COUNT}...")

        f.write("COMMIT TRANSACTION;\n")
    
    print(f"Success! Generated stress_data.surql with {COUNT} trades.")
    print("To import: surreal import --conn http://localhost:8000 --user root --pass root --ns traderlog --db main stress_data.surql")

if __name__ == "__main__":
    generate_trades()
