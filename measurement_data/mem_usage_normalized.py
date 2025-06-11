import os
import json
import pandas as pd

# Configuration
base_folder = "./execution_time"
payload_file = "execution_time_payloads.json"
environments = ["native", "wasm", "wasm-aot"]
categories = ["warm", "cold"]
BYTES_TO_MB = 1024 * 1024

# Load payloads
with open(payload_file, "r") as f:
    payloads = json.load(f)

def process_memory_csv(file_path):
    """Processes memory CSV in bytes and returns mean memory usage per payload in MB."""
    data = pd.read_csv(file_path, header=None, names=["Time", "Device", "Memory", "Type", "ID"], skip_blank_lines=False)
    data["Memory"] = data["Memory"] / BYTES_TO_MB  # Convert to MB

    empty_rows = data[data["Time"].isna()].index.tolist()
    while empty_rows and empty_rows[-1] == len(data) - 1:
        empty_rows.pop()

    data["Payload"] = None
    start_idx = 0
    for i, end_idx in enumerate(empty_rows + [len(data)]):
        data.loc[start_idx:end_idx-1, "Payload"] = i
        start_idx = end_idx + 1

    data.dropna(inplace=True)
    data["Memory"] = pd.to_numeric(data["Memory"])

    return data.groupby("Payload")["Memory"].mean()

# Process each function
for function_name in os.listdir(base_folder):
    function_path = os.path.join(base_folder, function_name)
    if not os.path.isdir(function_path):
        continue

    print(f"\n🧠 Processing memory for function: {function_name}")
    input_sizes = payloads.get(function_name, "").split(",")
    input_sizes = [s.strip() for s in input_sizes if s.strip()]
    raw_data = []

    for i, size in enumerate(input_sizes):
        row = {"InputSize": int(size)}
        for env in environments:
            for cat in categories:
                key = f"{env.capitalize()}-{cat.capitalize()}"
                csv_path = os.path.join(function_path, env, cat, "filtered_mem.csv")
                if os.path.exists(csv_path):
                    mean_values = process_memory_csv(csv_path)
                    if i < len(mean_values):
                        val = mean_values.values[i]
                        row[key] = val
                    else:
                        row[key] = None
                else:
                    row[key] = None
        raw_data.append(row)

    # Print formatted output
    keys = [f"{env.capitalize()}-{cat.capitalize()}" for env in environments for cat in categories]
    for key in keys:
        print(f"\n✅ {key}")
        coords = []
        for row in raw_data:
            val = row.get(key)
            if val is not None:
                coords.append(f"({row['InputSize']}, {val:.2f})")
        print(" ".join(coords))

