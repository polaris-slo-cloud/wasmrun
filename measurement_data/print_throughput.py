import os
import numpy as np
import pandas as pd

# Configuration
base_folder = "./throughput"  # Adjust if needed
rps_levels = np.array([10, 20, 30, 40, 50, 60, 70, 80, 100, 200])
runs = 5
environments = ["native", "wasm", "wasm-aot"]

def compute_latency_cdf(latencies, rps):
    sorted_indices = np.argsort(latencies)
    sorted_latencies = latencies[sorted_indices]
    sorted_rps = rps[sorted_indices]
    cum_rps = np.cumsum(sorted_rps)
    cum_rps_norm = cum_rps / cum_rps[-1]  # Normalize
    return sorted_latencies, cum_rps_norm

def process_latency_file(file_path):
    df = pd.read_csv(file_path, header=None, names=["ExecutionTime"], skip_blank_lines=False)
    df = df.dropna()
    if len(df) < len(rps_levels):
        raise ValueError(f"Not enough data in {file_path}. Expected {len(rps_levels)} values.")
    df = df.iloc[:len(rps_levels)]
    return df["ExecutionTime"].values.astype(float)

def generate_cdf_csv():
    for function in os.listdir(base_folder):
        function_dir = os.path.join(base_folder, function)
        if not os.path.isdir(function_dir):
            continue

        all_entries = []

        for env in environments:
            time_path = os.path.join(function_dir, env, "time.csv")
            if os.path.exists(time_path):
                try:
                    latencies = process_latency_file(time_path)
                    lat_sorted, cdf = compute_latency_cdf(latencies, rps_levels)
                    for latency, prob in zip(lat_sorted, cdf):
                        all_entries.append({"env": env, "latency_ms": latency, "cdf_normalized_rps": prob})
                except Exception as e:
                    print(f"Error in {function}/{env}: {e}")
                    continue

        # Write to CSV
        output_csv = os.path.join(function_dir, "cdf_values.csv")
        pd.DataFrame(all_entries).to_csv(output_csv, index=False)
        print(f"Written: {output_csv}")

generate_cdf_csv()
