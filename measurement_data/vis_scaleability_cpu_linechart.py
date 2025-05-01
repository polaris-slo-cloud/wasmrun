import os
import pandas as pd
import matplotlib.pyplot as plt

# Configuration
base_folder = "./throughput"
y_label = "CPU Usage (%)"
x_label = "Mean Execution Time (seconds)"

# Function to process CPU CSV files
def process_cpu_csv(file_path):
    with open(file_path, "r") as f:
        content = f.read().strip()  # Remove leading/trailing whitespace

    # Split runs by empty lines
    raw_runs = content.split("\n\n")  # Empty line separation
    runs = []

    for raw_run in raw_runs:
        lines = raw_run.strip().split("\n")  # Split into lines
        df = pd.DataFrame([line.split(",") for line in lines], columns=["timestamp", "node", "usage", "metric", "extra"])
        df["timestamp"] = df["timestamp"].astype(float)
        df["usage"] = df["usage"].astype(float)
        df = df[["timestamp", "node", "usage"]]
        runs.append(df)

    return runs

# Normalize timestamps within each run and compute execution time
def process_runs(runs):
    execution_times = []
    normalized_runs = []

    for df in runs:
        df = df.sort_values(by=["timestamp"])
        start_time = df["timestamp"].iloc[0]
        end_time = df["timestamp"].iloc[-1]
        execution_time = end_time - start_time

        df["relative_time"] = df["timestamp"] - start_time  # Normalize timestamps to start from 0
        execution_times.append(execution_time)
        normalized_runs.append(df)

    mean_execution_time = sum(execution_times) / len(execution_times) if execution_times else 0
    return normalized_runs, mean_execution_time

# Function to process and compare CPU usage
def process_cpu_comparison(base_folder):
    for function_name in os.listdir(base_folder):
        function_folder = os.path.join(base_folder, function_name)
        cpu_csv_path_wasm = os.path.join(function_folder, "wasm", "filtered_cpu.csv")
        cpu_csv_path_native = os.path.join(function_folder, "native", "filtered_cpu.csv")

        if not (os.path.exists(cpu_csv_path_wasm) or os.path.exists(cpu_csv_path_native)):
            print(f"Skipping function '{function_name}' as no filtered_cpu.csv file found.")
            continue

        print(f"Processing function '{function_name}' for CPU comparison...")

        wasm_runs, wasm_mean_exec_time = process_runs(process_cpu_csv(cpu_csv_path_wasm)) if os.path.exists(cpu_csv_path_wasm) else ([], 0)
        native_runs, native_mean_exec_time = process_runs(process_cpu_csv(cpu_csv_path_native)) if os.path.exists(cpu_csv_path_native) else ([], 0)

        mean_execution_time = (wasm_mean_exec_time + native_mean_exec_time) / 2 if wasm_runs and native_runs else (wasm_mean_exec_time or native_mean_exec_time)

        plot_cpu_usage(function_name, wasm_runs, native_runs, mean_execution_time, base_folder)

# Function to plot CPU usage per function
def plot_cpu_usage(function_name, wasm_runs, native_runs, mean_execution_time, base_folder):
    plt.figure(figsize=(12, 6))

    def aggregate_runs(runs, label):
        all_nodes = set()
        for df in runs:
            all_nodes.update(df["node"].unique())

        for node in all_nodes:
            run_data = [df[df["node"] == node] for df in runs if node in df["node"].values]

            # Interpolating to a common time scale
            common_time = pd.Series(range(int(mean_execution_time) + 1))  # X-Axis: Mean Execution Time steps
            interpolated_dfs = []
            for df in run_data:
                df = df.groupby("relative_time")["usage"].mean().reset_index()
                df = df.set_index("relative_time").reindex(common_time).interpolate().reset_index()
                interpolated_dfs.append(df)

            if interpolated_dfs:
                mean_usage = pd.concat(interpolated_dfs).groupby("index")["usage"].mean().reset_index()
                plt.plot(mean_usage["index"], mean_usage["usage"], marker='o', linestyle='-', label=f"{label} - {node}")

    if wasm_runs:
        aggregate_runs(wasm_runs, "WASM")
    if native_runs:
        aggregate_runs(native_runs, "Native")

    plt.xlabel(x_label)
    plt.ylabel(y_label)
    plt.title(f"CPU Usage Comparison for {function_name}")
    plt.legend()
    plt.grid()

    output_path = os.path.join("visualization", "scaleability", f"{function_name}_cpu.png")
    os.makedirs(os.path.dirname(output_path), exist_ok=True)

    plt.savefig(output_path)
    plt.close()
    print(f"Saved plot: {output_path}")

# Execute CPU processing
process_cpu_comparison(base_folder)

print("CPU usage comparison charts have been generated and saved.")
