import os
import pandas as pd
import matplotlib.pyplot as plt

# Configuration variables
base_folder = "./throughput"  # Update as needed
rps_values = [10, 20, 30, 40, 50, 60, 70, 80, 100, 200]  # RPS values used in tests
runs = 5  # Number of runs per RPS
y_label = "Avg. Latency (s)"
x_label = "Concurrent Requests"

def process_latency_csv(file_path, rps_values, runs, function_name, env):
    """
    Processes the latency CSV file, calculates median latency values for each RPS.
    
    Args:
        file_path (str): Path to the latency time CSV file.
        rps_values (list): List of RPS values.
        runs (int): Number of runs per RPS.
        function_name (str): Name of the function being processed.
        env (str): Environment (wasm/native) being processed.
    
    Returns:
        pandas.Series: Median latency values for each RPS.
    """
    print(f"Processing: Function='{function_name}', Environment='{env}', File='{file_path}'")
    
    # Read the CSV file assuming blank lines separate runs
    data = pd.read_csv(file_path, header=None, names=["ExecutionTime"], skip_blank_lines=False)
    
    # Identify empty rows separating run groups
    empty_rows = data[data["ExecutionTime"].isna()].index.tolist()
    
    # Remove trailing empty rows if any
    while empty_rows and empty_rows[-1] == len(data) - 1:
        empty_rows.pop()
    
    # Ensure number of runs per RPS matches the expected count
    number_of_runs = len(empty_rows) + 1
    if number_of_runs != runs:
        raise ValueError(f"Function='{function_name}', Environment='{env}' -> Expected {runs} runs, but found {number_of_runs} in {file_path}")
    
    # Assign RPS values to rows based on empty line indices
    data["RPS"] = None
    start_idx = 0
    for i, end_idx in enumerate(empty_rows + [len(data)]):
        section_length = end_idx - start_idx
        rps_subset = rps_values[: section_length]  # Ensure equal lengths
    
        # Check for mismatched lengths and truncate extra rows if necessary
        if len(rps_subset) < section_length:
            print(f"Warning: Truncating extra rows for function='{function_name}', environment='{env}', section={i + 1}")
            data = data.drop(index=range(start_idx + len(rps_subset), end_idx))
            end_idx = start_idx + len(rps_subset)
    
        if len(rps_subset) != (end_idx - start_idx):
            raise ValueError(f"Function='{function_name}', Environment='{env}', Section={i + 1} -> Mismatch in RPS values length. Expected: {len(rps_subset)}, Found: {section_length}")
    
        data.loc[start_idx:end_idx-1, "RPS"] = rps_subset
        start_idx = end_idx + 1
    
    # Drop empty rows
    data.dropna(inplace=True)
    
    # Convert execution times to numeric
    data["ExecutionTime"] = pd.to_numeric(data["ExecutionTime"])
    
    # Calculate Average Latency using the correct formula
    avg_latency_values = (data.groupby("RPS")["ExecutionTime"].mean() / data.groupby("RPS")["RPS"].count()) / 1000
    
    return avg_latency_values

def generate_latency_charts(base_folder, rps_values, runs):
    """
    Traverses function folders and generates latency comparison charts.
    
    Args:
        base_folder (str): The root folder containing throughput data.
        rps_values (list): List of RPS values used in the tests.
        runs (int): Number of runs per RPS.
    """
    for function_name in os.listdir(base_folder):
        function_folder = os.path.join(base_folder, function_name)
    
        time_csv_path_wasm = os.path.join(function_folder, "wasm", "time.csv")
        time_csv_path_native = os.path.join(function_folder, "native", "time.csv")
    
        if not (os.path.exists(time_csv_path_wasm) or os.path.exists(time_csv_path_native)):
            print(f"Skipping folder '{function_name}' as no time.csv file found.")
            continue
    
        plt.figure(figsize=(10, 6))
    
        environments = ["wasm", "native"]
        colors = {"wasm": "blue", "native": "red"}
    
        for env in environments:
            time_csv_path = os.path.join(function_folder, env, "time.csv")
            if os.path.exists(time_csv_path):
                try:
                    avg_latency_values = process_latency_csv(time_csv_path, rps_values, runs, function_name, env)
                    plt.plot(avg_latency_values.index, avg_latency_values.values, marker='o', linestyle='-', label=f"{env.capitalize()}", color=colors[env])
                except ValueError as e:
                    print(f"Error processing {function_name} in {env}: {e}")
                    continue
    
        plt.xlabel(x_label)
        plt.ylabel(y_label)
        plt.title(f"{function_name} - Latency Comparison")
    
        plt.xscale("log")
        plt.xticks(rps_values, labels=[str(rps) for rps in rps_values])
    
        
    
        plt.legend()
        plt.grid(True, linestyle='--', linewidth=0.5)
    
        # Save chart
        output_path = os.path.join("visualization","scaleability",f"{function_name}_latency.png")
        plt.savefig(output_path)
        plt.close()

# Generate latency comparison charts
generate_latency_charts(base_folder, rps_values, runs)

print("Latency comparison charts have been generated and saved.")
