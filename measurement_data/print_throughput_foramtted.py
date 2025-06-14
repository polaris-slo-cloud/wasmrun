import os
import numpy as np
import pandas as pd

# Configuration
base_folder = "./throughput"
rps_levels = np.array([10, 20, 30, 40, 50, 60, 70, 80, 100, 200])
environments = ["native", "wasm", "wasm-aot"]

def compute_latency_cdf(latencies, rps):
    sorted_indices = np.argsort(latencies)
    sorted_latencies = latencies[sorted_indices]
    sorted_rps = rps[sorted_indices]
    cum_rps = np.cumsum(sorted_rps)
    cum_rps_norm = cum_rps / cum_rps[-1]
    return sorted_latencies, cum_rps_norm

def process_latency_file(file_path):
    df = pd.read_csv(file_path, header=None, names=["ExecutionTime"], skip_blank_lines=False)
    df = df.dropna()
    if len(df) < len(rps_levels):
        raise ValueError(f"Not enough data in {file_path}. Expected {len(rps_levels)} values.")
    # Convert from milliseconds to seconds
    return df.iloc[:len(rps_levels)]["ExecutionTime"].values.astype(float) / 1000.0

def print_formatted_cdf():
    for function in os.listdir(base_folder):
        function_dir = os.path.join(base_folder, function)
        if not os.path.isdir(function_dir):
            continue

        print(f"\nFunction: {function}")
        for env in environments:
            time_path = os.path.join(function_dir, env, "time.csv")
            if os.path.exists(time_path):
                try:
                    latencies = process_latency_file(time_path)
                    sorted_latencies, cdf = compute_latency_cdf(latencies, rps_levels)

                    output = f"{env} ="
                    for latency, prob in zip(sorted_latencies, cdf):
                        output += f" ({latency:.3f}, {prob:.4f})"
                    print(output)

                except Exception as e:
                    print(f"Error in {function}/{env}: {e}")

print_formatted_cdf()

