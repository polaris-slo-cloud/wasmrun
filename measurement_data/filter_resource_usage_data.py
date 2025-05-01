import os
import csv

def extract_container_ids(file_path):
    """Extract container IDs from pod_id.csv."""
    valid_ids = set()
    with open(file_path, 'r') as f:
        for line in f:
            if line.startswith("Container ID:"):
                container_id = line.split("Container ID:")[1].strip()
                if container_id:
                    valid_ids.add(container_id)
    return valid_ids

def extract_intervals(info_file):
    """Extract time intervals from info.txt."""
    intervals = []
    with open(info_file, 'r') as f:
        start_time, end_time = None, None
        for line in f:
            if line.startswith("Start-time:"):
                start_time = float(line.split("Start-time:")[1].strip())
            elif line.startswith("End-time:"):
                end_time = float(line.split("End-time:")[1].strip())
            if start_time is not None and end_time is not None:
                intervals.append((start_time, end_time))
                start_time, end_time = None, None
    return intervals

def get_rows(file_path):
    rows = []
    with open(file_path, 'r') as f:
        reader = csv.reader(f)
        for row in reader:
            rows.append(row)  # Each row is a list of cell values
    return rows

def filter_invalid_entries(file_path, valid_ids):
    """Filter lines from a CSV file where the container ID is not valid."""
    rows = get_rows(file_path)
    filtered_lines = []
    
    for row in rows:
        if row:  # Ensure row is not empty
            container_id = row[-1]  # Assuming container ID is in the last column
            if container_id in valid_ids:
                filtered_lines.append(row)
                
    return filtered_lines

def filter_by_intervals(rows, intervals):
    """Filter lines from a list of rows to only include those within the intervals."""
    filtered_lines = []
    for start, end in intervals:
        # Filter rows within the current interval
        interval_rows = [row for row in rows if row and start <= float(row[0]) <= end]
        filtered_lines.extend(interval_rows)
        # Add an empty line as a separator
        filtered_lines.append([])
    return filtered_lines

def process_directory(root_dir):
    """Traverse directories and process files."""
    for subdir, _, files in os.walk(root_dir):

        if {'info.txt', 'mem.csv', 'cpu.csv'}.issubset(files):

            print(f"Processing directory: {subdir}")
            
            mem_path = os.path.join(subdir, 'mem.csv')
            cpu_path = os.path.join(subdir, 'cpu.csv')
            info_path = os.path.join(subdir, 'info.txt')

            # Extract intervals
            intervals = extract_intervals(info_path)

            if "throughput" in subdir:

                mem_data = get_rows(mem_path)
                cpu_data = get_rows(cpu_path)

            elif "execution_time" in subdir:

                if not ({'pod_id.csv',}.issubset(files)):
                    print("Error: File pod_id.csv missing in directory:'%s'" % subdir)
                    continue

                # Extract valid container IDs
                pod_id_path = os.path.join(subdir, 'pod_id.csv')
                valid_ids = extract_container_ids(pod_id_path)

                mem_data = filter_invalid_entries(mem_path, valid_ids)
                cpu_data = filter_invalid_entries(cpu_path, valid_ids)
            else:
                print("Unknown subdirectoy structure - must contain 'throughput' or 'execution_time'.")
                continue

            mem_filtered_by_intervals = filter_by_intervals(mem_data, intervals)
            cpu_filtered_by_intervals = filter_by_intervals(cpu_data, intervals)

            mem_output_path = os.path.join(subdir, 'filtered_mem.csv')
            with open(mem_output_path, 'w', newline='') as f:
                writer = csv.writer(f)
                writer.writerows(mem_filtered_by_intervals)

            cpu_output_path = os.path.join(subdir, 'filtered_cpu.csv')
            with open(cpu_output_path, 'w', newline='') as f:
                writer = csv.writer(f)
                writer.writerows(cpu_filtered_by_intervals)

            print(f"Filtered files saved in {subdir}")
            
        else:
            print("Path: '" + subdir + "' - Files to process are missing: info.txt'or 'mem.csv' or 'cpu.csv'")
            continue

if __name__ == "__main__":
    root_directory = "./execution_time/"
    #print(root_directory)
    process_directory(root_directory)
