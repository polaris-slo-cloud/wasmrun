import os
import json
import pandas as pd

# Configuration
base_folder = "./execution_time"
payload_file = "execution_time_payloads.json"
environments = ["native", "wasm"]
categories = ["cold", "warm"]
BYTES_TO_MB = 1024 * 1024

# Load payload values
with open(payload_file, "r") as f:
    payloads = json.load(f)

def process_memory_csv(file_path):
    """
    Processes memory CSV, splits by blank lines into payloads,
    and returns mean memory usage in MB per payload.
    """
    data = pd.read_csv(file_path, header=None, names=["Time", "Device", "Memory", "Type", "ID"], skip_blank_lines=False)
    data["Memory"] = data["Memory"] / BYTES_TO_MB  # bytes → MB

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

# Process all functions
for function_name in os.listdir(base_folder):
    function_path = os.path.join(base_folder, function_name)
    if not os.path.isdir(function_path):
        continue

    print(f"Processing function: {function_name}")
    input_sizes = payloads.get(function_name, "").split(",")
    input_sizes = [s.strip() for s in input_sizes if s.strip()]
    rows = []

    local_data = []

    for i, size in enumerate(input_sizes):
        row = {"InputSize": size}
        for env in environments:
            for cat in categories:
                label = f"{env.capitalize()}-{cat.capitalize()}"
                csv_path = os.path.join(function_path, env, cat, "filtered_mem.csv")
                if os.path.exists(csv_path):
                    values = process_memory_csv(csv_path)
                    if i < len(values):
                        row[label] = values.values[i]
                    else:
                        row[label] = None
                else:
                    row[label] = None
        local_data.append(row)

    df = pd.DataFrame(local_data)
    output_csv = os.path.join(function_path, "memory_summary.csv")
    df.to_csv(output_csv, index=False)
    print(f"Saved: {output_csv}")

