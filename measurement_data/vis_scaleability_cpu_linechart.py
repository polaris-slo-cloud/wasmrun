import os
import pandas as pd
import matplotlib.pyplot as plt

# Configuration
base_folder = "./throughput"
y_label = "CPU Usage (%)"
x_label = "Time (seconds)"

# Function to process cpu CSV files
def process_cpu_csv(file_path):
    df = pd.read_csv(file_path, header=None, names=["timestamp", "node", "usage", "metric", "extra"])
    df["timestamp"] = df["timestamp"].astype(float)
    df = df[["timestamp", "node", "usage"]]
    return df

# Normalize timestamps within each run
def normalize_timestamps(df):
    df = df.sort_values(by=["timestamp"])
    df["relative_time"] = df["timestamp"] - df["timestamp"].iloc[0]
    return df

# Function to process and compare cpu usage
def process_cpu_comparison(base_folder):
    for function_name in os.listdir(base_folder):
        function_folder = os.path.join(base_folder, function_name)
        mem_csv_path_wasm = os.path.join(function_folder, "wasm", "filtered_cpu.csv")
        mem_csv_path_native = os.path.join(function_folder, "native", "filtered_cpu.csv")

        if not (os.path.exists(mem_csv_path_wasm) or os.path.exists(mem_csv_path_native)):
            print(f"Skipping function '{function_name}' as no filtered_cpu.csv file found.")
            continue

        print(f"Processing function '{function_name}' for cpu comparison...")

        wasm_data = process_cpu_csv(mem_csv_path_wasm) if os.path.exists(mem_csv_path_wasm) else None
        native_data = process_cpu_csv(mem_csv_path_native) if os.path.exists(mem_csv_path_native) else None

        if wasm_data is not None:
            wasm_runs = [normalize_timestamps(run) for run in identify_runs(wasm_data)]
            mean_wasm = pd.concat(wasm_runs).groupby(["relative_time", "node"])["usage"].mean().reset_index()
        else:
            mean_wasm = None

        if native_data is not None:
            native_runs = [normalize_timestamps(run) for run in identify_runs(native_data)]
            mean_native = pd.concat(native_runs).groupby(["relative_time", "node"])["usage"].mean().reset_index()
        else:
            mean_native = None

        plot_memory_usage(mean_wasm, mean_native, function_name, base_folder)

# Identify runs by detecting gaps in timestamps
def identify_runs(df, gap_threshold=5):
    runs = []
    current_run = []
    for _, row in df.iterrows():
        if len(current_run) > 0 and row["timestamp"] - current_run[-1]["timestamp"] > gap_threshold:
            runs.append(pd.DataFrame(current_run))
            current_run = []
        current_run.append(row)
    if len(current_run) > 0:
        runs.append(pd.DataFrame(current_run))
    return runs

# Function to plot memory usage
def plot_memory_usage(mean_wasm, mean_native, function_name, base_folder):
    plt.figure(figsize=(10, 5))
    if mean_wasm is not None:
        for node in mean_wasm["node"].unique():
            node_data = mean_wasm[mean_wasm["node"] == node]
            plt.plot(node_data["relative_time"], node_data["usage"], marker='o', linestyle='-', label=f"WASM - {node}")
    
    if mean_native is not None:
        for node in mean_native["node"].unique():
            node_data = mean_native[mean_native["node"] == node]
            plt.plot(node_data["relative_time"], node_data["usage"], marker='o', linestyle='-', label=f"Native - {node}")
    
    plt.xlabel(x_label)
    plt.ylabel(y_label)
    plt.title(f"CPU Usage Comparison for {function_name}")
    plt.legend()
    plt.grid()

    output_path = os.path.join("visualization","scaleability",f"{function_name}_cpu.png")
    plt.savefig(output_path)
    plt.close()

# Execute cpu processing
process_cpu_comparison(base_folder)

print("CPU usage comparison charts have been generated and saved.")
