import os
import json
import pandas as pd
import matplotlib.pyplot as plt

base_folder = "./execution_time"
y_label = "Execution Time (ms)"
x_label = "Payload"

# Load payload labels from JSON
with open("execution_time_payloads.json", "r") as json_file:
    payload_data = json.load(json_file)

# Function to process CSV files and extract median execution times
def process_time_csv(file_path):
    data = pd.read_csv(file_path, header=None, names=["Function", "Time"], skip_blank_lines=False)
    empty_rows = data[data["Function"].isna()].index.tolist()

    while empty_rows and empty_rows[-1] == len(data) - 1:
        empty_rows.pop()

    number_of_payloads = len(empty_rows) + 1
    data["Payload"] = None
    start_idx = 0

    for i, end_idx in enumerate(empty_rows + [len(data)]):
        data.loc[start_idx:end_idx-1, "Payload"] = i + 1  # Assigning index-based payload values
        start_idx = end_idx + 1

    data.dropna(inplace=True)

    # ms
    data["Time"] = pd.to_numeric(data["Time"])# / 1000  

    median_times = data.groupby("Payload")["Time"].mean()
    return median_times, number_of_payloads

# Function to generate charts
def generate_charts(base_folder):
    for function_name, payload_values in payload_data.items():
        function_folder = os.path.join(base_folder, function_name)
        if not os.path.isdir(function_folder):
            continue

        environments = ["wasm", "native"]
        categories = ["cold", "warm"]

        plt.figure(figsize=(8, 6))

        for category in categories:
            for env in environments:
                time_csv_path = os.path.join(function_folder, env, category, "time.csv")
                if os.path.exists(time_csv_path):
                    print(f"Processing: Function='{function_name}', Environment='{env}', Category='{category}'")

                    median_times, number_of_payloads = process_time_csv(time_csv_path)

                    linestyle = "--" if category == "cold" else "-"

                    plt.plot(
                        median_times.index, 
                        median_times.values, 
                        label=f"{env.capitalize()} - {category.capitalize()}",
                        marker='o',
                        linestyle=linestyle
                    )

        # Set X-axis labels based on JSON-defined payload values
        if payload_values:
            payload_labels = payload_values.split(",")
        else:
            payload_labels = list(range(1, number_of_payloads + 1))  # Fallback if no values in JSON

        # Ensure labels match the number of detected payloads
        plt.xticks(ticks=range(1, len(payload_labels) + 1), labels=payload_labels[:number_of_payloads])

        # Axis labels & title
        plt.yscale('log')
        plt.xlabel(x_label)
        plt.ylabel(y_label)
        plt.title(f"{function_name} - Execution time comparison")

        # Legend & Grid
        plt.legend()
        plt.grid(True, which="both", linestyle='--', linewidth=0.5)


        output_path = os.path.join("visualization", "execution_time", f"{function_name}.png")
        os.makedirs(os.path.dirname(output_path), exist_ok=True)

        plt.savefig(output_path)
        plt.close()

generate_charts(base_folder)

print("Charts have been generated and saved in the respective function folders.")
