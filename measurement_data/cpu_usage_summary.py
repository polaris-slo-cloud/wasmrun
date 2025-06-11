import os
import json
import pandas as pd

# Configuration
base_folder = "./execution_time"
payload_file = "execution_time_payloads.json"
environments = ["native", "wasm", "wasm-aot"]
categories = ["warm", "cold"]
CPU_HZ = 2_400_000_000  # 2.4 GHz

# Load payload values
with open(payload_file, "r") as f:
    payloads = json.load(f)

def process_cpu_csv(file_path):
    """Processes CPU telemetry in nanoseconds and converts to usage percentage using CPU Hz."""
    data = pd.read_csv(file_path, header=None, names=["Time", "Device", "CPU", "Type", "ID"], skip_blank_lines=False)

    # Identify payload segments using blank lines
    empty_rows = data[data["Time"].isna()].index.tolist()
    while empty_rows and empty_rows[-1] == len(data) - 1:
        empty_rows.pop()

    data["Payload"] = None
    start_idx = 0
    for i, end_idx in enumerate(empty_rows + [len(data)]):
        data.loc[start_idx:end_idx-1, "Payload"] = i
        start_idx = end_idx + 1

    data.dropna(inplace=True)
    data["CPU"] = pd.to_numeric(data["CPU"])
    data["CPU"] = (data["CPU"] / CPU_HZ) * 100
    return data.groupby("Payload")["CPU"].mean()

# Iterate over all functions
for function_name in os.listdir(base_folder):
    function_path = os.path.join(base_folder, function_name)
    if not os.path.isdir(function_path):
        continue

    print(f"Processing function: {function_name}")
    input_sizes = payloads.get(function_name, "").split(",")
    input_sizes = [s.strip() for s in input_sizes if s.strip()]
    rows = []

    # Collect raw CPU values for this function to find local max
    local_values = []

    raw_data = []  # For second pass

    for i, size in enumerate(input_sizes):
        row = {"InputSize": size}
        for env in environments:
            for cat in categories:
                key = f"{env.capitalize()}-{cat.capitalize()}"
                csv_path = os.path.join(function_path, env, cat, "filtered_cpu.csv")
                if os.path.exists(csv_path):
                    mean_values = process_cpu_csv(csv_path)
                    if i < len(mean_values):
                        val = mean_values.values[i]
                        row[key] = val
                        local_values.append(val)
                    else:
                        row[key] = None
                else:
                    row[key] = None
        raw_data.append(row)

    # Find local max per function
    if not local_values:
        print(f"No data for {function_name}, skipping.")
        continue
    local_max = max(local_values)

    # Normalize using local max
    for row in raw_data:
        for key in row:
            if key != "InputSize" and row[key] is not None:
                row[key] = row[key] / local_max
        rows.append(row)

    df = pd.DataFrame(rows)
    output_csv = os.path.join(function_path, "cpu_summary.csv")
    df.to_csv(output_csv, index=False)
    print(f"Saved normalized CSV: {output_csv}")

