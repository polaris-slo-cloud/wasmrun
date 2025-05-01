import os
import json
import pandas as pd
import matplotlib.pyplot as plt

# Configuration variables
base_folder = "./execution_time"
y_label = "CPU Usage (%)"
x_label = "Payload"

# Load payload values from an external JSON file
payload_file = "execution_time_payloads.json"
with open(payload_file, "r") as f:
    payloads = json.load(f)

def process_cpu_csv(file_path):
    """
    Processes the CPU usage CSV file, calculates median values for each payload,
    and converts CPU usage to percentage.
    """
    data = pd.read_csv(file_path, header=None, names=["Time", "Device", "CPU", "Type", "ID"], skip_blank_lines=False)
    data["CPU"] = data["CPU"] * 100  # Convert CPU fraction to percentage

    empty_rows = data[data["Time"].isna()].index.tolist()
    while empty_rows and empty_rows[-1] == len(data) - 1:
        empty_rows.pop()

    number_of_payloads = len(empty_rows) + 1
    data["Payload"] = None
    start_idx = 0

    for i, end_idx in enumerate(empty_rows + [len(data)]):
        data.loc[start_idx:end_idx-1, "Payload"] = i + 1
        start_idx = end_idx + 1

    data.dropna(inplace=True)
    data["CPU"] = pd.to_numeric(data["CPU"])

    median_cpu = data.groupby("Payload")["CPU"].mean()
    return median_cpu, number_of_payloads

def generate_cpu_chart(base_folder):
    for function_name in os.listdir(base_folder):
        function_folder = os.path.join(base_folder, function_name)
        if not os.path.isdir(function_folder):
            continue

        plt.figure(figsize=(10, 6))
        
        environments = ["wasm", "native"]
        categories = ["cold", "warm"]
        colors = {"wasm_cold": "blue", "wasm_warm": "cyan", "native_cold": "red", "native_warm": "orange"}

        for env in environments:
            for category in categories:
                cpu_csv_path = os.path.join(function_folder, env, category, "filtered_cpu.csv")
                if os.path.exists(cpu_csv_path):
                    print(f"Processing CPU: Function='{function_name}', Environment='{env}', Category='{category}'")
                    median_cpu, number_of_payloads = process_cpu_csv(cpu_csv_path)
                    label = f"{env.capitalize()} - {category.capitalize()}"
                    
                    # Get the predefined payload values
                    payload_values = payloads.get(function_name, "").split(",")
                    if payload_values and payload_values[0] != "":
                        try:
                            payload_values = [float(p) for p in payload_values if p.strip()]
                            if len(payload_values) == len(median_cpu):
                                x_axis_values = payload_values
                            else:
                                print(f"Warning: Mismatch in payload length for {function_name}. Using default indices.")
                                x_axis_values = list(range(1, len(median_cpu) + 1))
                        except ValueError:
                            print(f"Warning: Invalid payload values for {function_name}")
                            x_axis_values = list(range(1, len(median_cpu) + 1))
                    else:
                        x_axis_values = list(range(1, len(median_cpu) + 1))
                    
                    print(f"Function: {function_name}, X-Axis Values: {x_axis_values}, Median CPU Values: {median_cpu.values}")  # Debugging statement
                    plt.plot(x_axis_values, median_cpu.values, marker='o', linestyle='-', label=label, color=colors[f"{env}_{category}"])
                    plt.xticks(x_axis_values, labels=[str(int(x)) if x.is_integer() else str(x) for x in x_axis_values])

        plt.xlabel(x_label)
        plt.ylabel(y_label)
        plt.title(f"{function_name} - CPU Usage Comparison")
        plt.legend()
        plt.grid(True, linestyle='--', linewidth=0.5)

        output_path = os.path.join("visualization", "execution_time", f"{function_name}_cpu.png")
        os.makedirs(os.path.dirname(output_path), exist_ok=True)

        plt.savefig(output_path)
        plt.close()

# Generate CPU usage charts
generate_cpu_chart(base_folder)

print("CPU usage comparison charts have been generated and saved.")
