#!/bin/bash

# load config
source config.sh

# options
clear_files_on_run="true"
runs=5
rps=(10 20 30 40 50 60 70 80 100 200)

# functions
declare -A functions=(
  #["hello-world"]=""
  ["fibonacci"]='{"number": 10000}'
  #["audio-generation"]='{"audio_length": 10, "storage_type": "memory", "path": "file.wav"}'
  ["image-resize"]='{"scale_factor": 1.5, "storage_type": "memory", "path": "file.jpg"}'
  #["fuzzy-search"]='{"search_term": "apple"}'
  #["get-prime-numbers"]='{"limit": 10000}'
  #["language-detection"]='{"input_text": "Hallo ich bin ein Mensch der Deutsch spricht."}'
  #["planet-system-simulation"]='{"n": 1000}'
  #["encrypt-message"]='{"key": "sWXKbbmxgkqmWVsJYNgt9dApYfQpwVDK", "message": "This message should get encrypted."}'
  #["decrypt-message"]='{"key": "sWXKbbmxgkqmWVsJYNgt9dApYfQpwVDK", "encrypted_data": "OvfNe80QhCK3xOBa5voAkonEhTuAyXTBFPsYJ5Rl9V7LbskmcDjrn2H0MSsxicnMqKyj"}'
  #["create-mandelbrot-bitmap"]='{"image_size": 16, "storage_type": "memory", "output_path": "file.pbm"}'
)

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

root_path="$HOME/telemetry"
throughput_path="$root_path/throughput"

measure_resources() {
  local dir_structure="$1"

  #Telemc in background
  echo "Setting up telemc-py." | tee -a $file_logs

  ${HOME}/telemc-py/.venv/bin/python ${script_dir}/telemd/rec_resource_usage.py $dir_structure "throughput" &
  PYTHON_PID=$!

  if ps -p $PYTHON_PID >/dev/null; then
    echo "Python script is running with PID: $PYTHON_PID" | tee -a $file_logs
  else
    echo "Failed to start Python script." | tee -a $file_logs
    exit 1
  fi
}

delete_all_ksvc() {
  function_names=$(echo "${!functions[@]}" | tr ' ' '|')
  echo "Searching for ksvc matching function names: $function_names" | tee -a $file_logs

  # Grab all ksvcs containing any of the function names
  ksvcs=$(microk8s kubectl get ksvc --no-headers | grep -E "$function_names" | awk '{print $1}')

  if [[ -n "$ksvcs" ]]; then
    echo "Found the following ksvc to delete:" | tee -a $file_logs
    echo "$ksvcs" | tee -a $file_logs

    for ksvc in $ksvcs; do
      echo "Deleting ksvc: $ksvc" | tee -a $file_logs
      microk8s kubectl delete ksvc "$ksvc"

      sleep 10

      pod=$(microk8s kubectl get pod --no-headers | grep -E "$ksvc" | awk '{print $1}')
      microk8s kubectl delete pods "$pod" --force --grace-period=0

      # Confirm deletion
      deleted=$(microk8s kubectl get pods --no-headers | grep -c "$pod")
      if [[ "$deleted" -ne 0 ]]; then
        echo "Pod $pod failed to delete. Retrying..." | tee -a $file_logs
        sleep 5
        microk8s kubectl delete pod "$pod" --wait=true
      else
        echo "Pod $pod deleted successfully." | tee -a $file_logs
      fi
    done

  else
    echo "No matching pods found for function names." | tee -a $file_logs
  fi
}

deploy_ksvc() {
  local function_name="$1"
  local runtime="$2"

  ./deploy.sh $function_name $runtime warm

  sleep 30
}

spin_up_pod() {
  local function_name="$1"
  local runtime="$2"
  local curl_cmd_string="$3"

  # Restore curl_cmd as an array
  eval "curl_cmd=($curl_cmd_string)"

  local pods=$(microk8s kubectl get pods --no-headers --field-selector status.phase=Running | grep "^${function_name}-${runtime}" | awk '{print $1}')

  if [[ -z "$pods" ]]; then
    # Spin up pod for warms start
    local max_retries=3
    local response=""
    local success=false
    for ((attempt = 1; attempt <= max_retries; attempt++)); do
      response=$("${curl_cmd[@]}" -o /dev/null -w "%{http_code}")
      if [[ "$response" == "200" ]]; then

        local started_pod=$(microk8s kubectl get pods --no-headers | grep "^${function_name}-${runtime}" | awk '{print $1}')
        microk8s kubectl wait --for=jsonpath='{.status.phase}'=Running pod/$started_pod

        success=true
        break
      else
        sleep "2"
      fi
    done

    if [[ "$success" == false ]]; then
      echo "Failed to get a successful response after $max_retries attempts." | tee -a $file_logs
      return 1
    fi
  else
    echo "Pod for function: $function_name-$runtime" already running. | tee -a $file_logs
  fi
}

throughput() {
  local function_name="$1"
  local runtime="$2"
  local run_index="$3"

  local payload="${functions[$function_name]}"
  header_host="Host: $function_name-$runtime.default.svc.cluster.local"
  header_content="Content-Type: application/json"

  local -a curl_cmd
  if [[ -n "$payload" ]]; then
    curl_cmd=(curl -s -X POST "$GATEWAY_URL" -H "$header_host" -H "$header_content" -d "$payload")
  else
    curl_cmd=(curl -s -X GET "$GATEWAY_URL" -H "$header_host" -H "$header_content")
  fi

  local curl_cmd_string=$(printf "%q " "${curl_cmd[@]}")

  # Shared file for storing results
  local result_file="/tmp/throughput_results_$run_index.log"
  >"$result_file"

  #Spin up pods to ensure warm start
  #spin_up_pod "$function_name" "$runtime" "$curl_cmd_string"

  send_request() {
    local function_name="$1"
    local result_file="$2"
    local curl_cmd_string="$3"

    # Restore curl_cmd as an array
    eval "curl_cmd=($curl_cmd_string)"

    local response

    local start_time=$(date +%s%N)
    response=$("${curl_cmd[@]}" -o /dev/null -w "%{http_code}")
    local end_time=$(date +%s%N)

    local elapsed_time=$(((end_time - start_time) / 1000000)) #ms

    if [[ "$response" == "200" ]]; then
      echo "$elapsed_time" >>"$result_file"
    else
      echo "Response FAILURE" | tee -a $file_logs
    fi

  }
  export -f send_request

  echo "Payload: $payload" >>"$file_info"

  echo "Start-time: $(date +%s).$(date +%N)" >>"$file_info"

  for concurrency in "${rps[@]}"; do
    echo "$function_name - testing concurrency level: $concurrency" | tee -a $file_logs

    # Clear results file
    >"$result_file"

    seq "$concurrency" | xargs -n1 -P"$concurrency" bash -c "send_request '$function_name' '$result_file' '$curl_cmd_string'"

    # Check if the number of lines matches the concurrency level
    line_count=$(wc -l <"$result_file")
    if [[ "$line_count" -ne "$concurrency" ]]; then
      echo "Warning: Expected $concurrency requests, but only $line_count completed." | tee -a $file_logs
    fi

    total_time=$(awk '{sum += $1} END {print sum}' "$result_file")
    echo "$total_time" >>"$file_time"

    echo "Completed testing for concurrency level: $concurrency" | tee -a $file_logs
  done

  ## Find pods starting with function_name and log their details
  microk8s kubectl get pods --no-headers -o custom-columns=":metadata.name" | grep "^${function_name}-${runtime}" | while read -r pod_name; do
    echo "Pod Name (${function_name}-${runtime}): $pod_name" >>"$file_info"
    microk8s kubectl describe pod "$pod_name" | grep "Container ID" | head -n 1 | awk '{print $3}' | sed 's|containerd://||' | sed 's|^|Container ID: |' >>"$file_info"

    #microk8s kubectl describe pod "$pod_name" | grep "Container ID" | head -n 1 | awk '{print $3}' | sed 's|containerd://||' | sed 's|^|Container ID: |'

  done

  echo "End-time: $(date +%s).$(date +%N)" >>"$file_info"

  echo "-----" >>"$file_info"
}

process_throughput_runs() {
  local function="$1"
  local runtime="$2"

  local current_path="$root_path/throughput/$function/$runtime"

  mkdir -p "$current_path"

  file_info="$current_path/info.txt"
  file_logs="$current_path/logs.txt"
  file_time="$current_path/time.csv"
  file_mem="$current_path/mem.csv"
  file_cpu="$current_path/cpu.csv"

  touch "$file_info"
  touch "$file_logs"
  touch "$file_mem"
  touch "$file_cpu"
  touch "$file_time"

  measure_resources "telemetry/throughput/$function/$runtime"

  echo "-----" >>"$file_info"

  local i
  for ((i = 1; i <= $runs; i++)); do

    echo "Run: $i" >>"$file_info"
    throughput "$function" "$runtime" "$i"

    echo "" >>"$file_time"
  done

  echo "Attempting to terminate the Python script with PID: $PYTHON_PID..." | tee -a $file_logs
  kill $PYTHON_PID
  sleep 1
  if ps -p $PYTHON_PID >/dev/null; then
    echo "Failed to terminate Python script with PID: $PYTHON_PID" | tee -a $file_logs
  else
    echo "Python script with PID $PYTHON_PID terminated successfully." | tee -a $file_logs
  fi

}

if [ "$#" -lt 1 ]; then
  echo "Usage: <function> [runtime]"
  echo "Available functions:"
  printf "  all\n"
  printf "  %s\n" "${!functions[@]}"
  echo "Available runtime options:"
  echo "  all"
  echo "  native"
  echo "  wasm"
  exit 1
fi
function=${1:-"all"}
runtime=${2:-"all"}

if [[ "$clear_files_on_run" == "true" ]]; then
  echo "You have set clear_files_on_run=true."
  echo "Press Enter to proceed, or Ctrl+C to cancel."
  read -r user_input
  if [[ -n "$user_input" ]]; then
    echo "Operation aborted."
    exit 1
  fi
  echo "Proceeding with the operation..."
else
  echo "clear_files_on_run is not set to true, skipping confirmation."
fi

#directory structure
if [[ "$clear_files_on_run" == "true" ]]; then
  echo "Removing the throughput directory and its contents..."
  rm -rf "$throughput_path"
else
  echo "Directory not removed, as clear_files_on_run is not true."
fi

mkdir $root_path

if [[ "$function" == "all" ]]; then
  for func in "${!functions[@]}"; do
    if [[ "$runtime" == "all" ]]; then
      for run in "native" "wasm"; do
        delete_all_ksvc

        deploy_ksvc "$func" "$run"

        echo "Running scaleability test for $func with $run runtime" | tee -a $file_logs
        process_throughput_runs "$func" "$run"
      done
    else
      delete_all_ksvc

      deploy_ksvc "$func" "$runtime"

      echo "Running scaleability test for $func with $run runtime" | tee -a $file_logs
      process_throughput_runs "$func" "$runtime"

    fi
  done
else

  if [[ " ${!functions[@]} " =~ " $function " ]]; then
    if [[ "$runtime" == "all" ]]; then
      for run in "native" "wasm"; do

        delete_all_ksvc

        deploy_ksvc "$function" "$run"

        echo "Running scaleability test for $function with $run runtime" | tee -a $file_logs
        process_throughput_runs "$function" "$run"
      done
    else
      delete_all_ksvc

      deploy_ksvc "$function" "$runtime"

      echo "Running scaleability test for $function with $runtime runtime" | tee -a $file_logs
      process_throughput_runs "$function" "$runtime"
    fi
  else
    echo "Unknown function: $function"
    exit 1
  fi
fi

echo "Measure script completed."
