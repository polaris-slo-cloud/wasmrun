#!/bin/bash

# load config
source config.sh

# options
clear_files_on_run="false"
runs=1

# functions
declare -A functions=(
  ["hello-world"]=""
  ["fibonacci"]='{"number": <VALUE>}' #cold-10s
  ["audio-generation"]='{"audio_size": <VALUE>, "storage_type": "memory", "path": "audio-generation:file_<VALUE>.wav"}' #cold-60s
  ["image-resize"]='{"scale_factor": <VALUE>, "storage_type": "memory", "path": "image-resize:file_<VALUE>.jpg"}' #cold-60s ksvc deployment takes time..
  ["fuzzy-search"]='{"search_term": "apple", "storage_type": "memory", "input_path": "files:search_text_<VALUE>kb.txt"}' #cold-60s
  ["get-prime-numbers"]='{"limit": <VALUE>}' #cold-60s
  ["language-detection"]='{"storage_type": "memory", "input_path": "files:spanish_text_<VALUE>kb.txt"}' #cold-60s
  ["planet-system-simulation"]='{"n": <VALUE>}' #60s - last payload about 5m
  ["encrypt-message"]='{"key" : "sWXKbbmxgkqmWVsJYNgt9dApYfQpwVDK", "storage_type": "memory", "input_path": "files:spanish_text_<VALUE>kb.txt"}' #cold-60s
  ["decrypt-message"]='{"key" : "sWXKbbmxgkqmWVsJYNgt9dApYfQpwVDK", "storage_type": "memory", "input_path": "files:encrypted_text_<VALUE>kb.txt"}' #cold-60s
  ["create-mandelbrot-bitmap"]='{"image_size": <VALUE>, "storage_type": "memory", "output_path": "create-mandelbrot-bitmap:file_<VALUE>.pbm"}' #cold-4m20s
)

# payloads
declare -A payloads
payloads["hello-world"]=""
payloads["fibonacci"]="10000,100000,1000000,2500000"
payloads["audio-generation"]="1,50,512,1024,2048"
payloads["image-resize"]="1.5,2.0,2.5,3.0"
payloads["fuzzy-search"]="1,50,512,1024,2048"
payloads["get-prime-numbers"]="10000,100000,1000000,2500000"
payloads["language-detection"]="1,50,512,1024,2048"
payloads["planet-system-simulation"]="10000,100000,1000000,2500000"
payloads["encrypt-message"]="1,50,512,1024,2048"
payloads["decrypt-message"]="1,50,512,1024,2048"
payloads["create-mandelbrot-bitmap"]="128,256,512,1024,2048"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

root_path="$HOME/telemetry"
execution_time_path="$root_path/execution_time"

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

delete_all_active_pods() {
  function_names=$(echo "${!functions[@]}" | tr ' ' '|')
  echo "Searching for all pods matching function names: $function_names" | tee -a $file_logs

  pods=$(microk8s kubectl get pods --no-headers | grep -E "^(${function_names})-" | awk '{print $1}')

  if [[ -n "$pods" ]]; then
    echo "Found the following pods to delete:" | tee -a $file_logs
    echo "$pods" | tee -a $file_logs

     for pod in $pods; do
      echo "Deleting pod: $pod" | tee -a $file_logs
      microk8s kubectl delete pod "$pod" --wait=false

      microk8s kubectl delete pod "$pod" --force

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
    echo "No matching pods found for function names" | tee -a $file_logs
  fi
}

deploy_ksvc() {
  local function_name="$1"
  local runtime="$2"
  local startup_type="$3"

  ./deploy.sh $function_name $runtime $startup_type

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
          sleep 2
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

delete_pod() {
  local function_name="$1"
  local runtime="$2"

  local pods=$(microk8s kubectl get pods --no-headers | grep "^${function_name}-${runtime}" | awk '{print $1}')

  if [[ -n "$pods" ]]; then
    echo "Deleting pods for function: $function_name-$runtime..." | tee -a $file_logs
    
    for pod in $pods; do
      echo "Deleting pod: $pod" | tee -a $file_logs
      microk8s kubectl delete pod "$pod" --wait=false

      microk8s kubectl delete pod "$pod" --force

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
    echo "No pods found for function: $function_name-$runtime" | tee -a $file_logs
  fi
}

measure_resources() {
  local dir_structure="$1"

  #Telemc in background
  echo "Setting up telemc-py." | tee -a $file_logs

  ${HOME}/telemc-py/.venv/bin/python ${script_dir}/telemd/rec_resource_usage.py $dir_structure "execution_time" &
  PYTHON_PID=$!

  if ps -p $PYTHON_PID >/dev/null; then
    echo "Python script is running with PID: $PYTHON_PID" | tee -a $file_logs
  else
    echo "Failed to start Python script." | tee -a $file_logs
    exit 1
  fi
}

execution_time() {
  local function_name="$1"
  local runtime="$2"
  local cold_start="$3"
  local payload="$4"

  local header_host="Host: $function_name-$runtime.default.svc.cluster.local"
  local header_content="Content-Type: application/json"

  local -a curl_cmd
  if [[ -n "$payload" ]]; then
    curl_cmd=(curl -s -X POST "$GATEWAY_URL" -H "$header_host" -H "$header_content" -d "$payload")
  else
    curl_cmd=(curl -s -X GET "$GATEWAY_URL" -H "$header_host" -H "$header_content")
  fi

  #spin up or delete pods for function
  if [[ "$cold_start" == "true" ]]; then
    #delete_pod "$function_name" "$runtime"

    echo "Waiting for pod $function_name-$runtime to terminate.."
    while [ -n "$(microk8s kubectl get pods | grep "^${function_name}-${runtime}")" ]
    do 
      sleep 0.5
    done

  else
    local curl_cmd_string=$(printf "%q " "${curl_cmd[@]}")
    spin_up_pod "$function_name" "$runtime" "$curl_cmd_string"
  fi

  local start_time=$(date +%s%N)
  full_response=$(mktemp)
  response=$("${curl_cmd[@]}" -o "$full_response" -w "%{http_code}")
  local end_time=$(date +%s%N)
  local elapsed_time=$(((end_time - start_time) / 1000000)) #ms

  microk8s kubectl get pods --no-headers -o custom-columns=":metadata.name" | grep "^${function_name}-${runtime}" | while read -r pod_name; do
    echo "Pod Name (${function_name}-${runtime}): $pod_name" >>"$file_info"
    microk8s kubectl describe pod "$pod_name" | grep "Container ID" | head -n 1 | awk '{print $3}' | sed 's|containerd://||' | sed 's|^|Container ID: |' >>"$file_pod_id"
    #microk8s kubectl describe pod "$pod_name" | grep "Container ID" | head -n 1 | awk '{print $3}' | sed 's|containerd://||'
  done

  #write to file
  if [[ "$response" == "200" ]]; then
    msg="$function_name-$runtime,$elapsed_time"
    echo "$msg" >>"$file_time"

    data_retrieval=$(jq -r '.data.data_retrieval // "N/A"' "$full_response")
    serialization=$(jq -r '.data.serialization // "N/A"' "$full_response")

    echo "$function_name-$runtime,data_retrieval:$data_retrieval,serialization:$serialization" >>"$file_metrics"

  else
    #error
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    msg="[$timestamp] Request failed for $function_name. HTTP code: $response"
    echo "$msg" | tee -a $file_logs
  fi

  rm -f "$full_response"
}

process_execution_time_runs() {
  local function="$1"
  local runtime="$2"
  local cold_start="${3:-false}"

  if [[ "$cold_start" == "true" ]]; then
    echo "Cold start is enabled." | tee -a $file_logs
    local current_path="$execution_time_path/$function/$runtime/cold"

  else
    echo "Warm start is enabled." | tee -a $file_logs
    local current_path="$execution_time_path/$function/$runtime/warm"
  fi

  mkdir -p "$current_path"

  file_metrics="$current_path/metrics.csv"
  file_info="$current_path/info.txt"
  file_logs="$current_path/logs.txt"
  file_pod_id="$current_path/pod_id.csv"
  file_time="$current_path/time.csv"
  file_mem="$current_path/mem.csv"
  file_cpu="$current_path/cpu.csv"

  touch "$file_metrics"
  touch "$file_info"
  touch "$file_logs"
  touch "$file_pod_id"
  touch "$file_time"
  touch "$file_mem"
  touch "$file_cpu"

  if [[ "$cold_start" == "true" ]]; then
    measure_resources "telemetry/execution_time/$function/$runtime/cold"
  else
    measure_resources "telemetry/execution_time/$function/$runtime/warm"
  fi

  echo "-----" >>"$file_info"

  if [[ -n "${payloads[$function]}" ]]; then
    IFS=',' read -r -a payload_array <<<"${payloads[$function]}"
    for value in "${payload_array[@]}"; do
      echo "Processing payload: $value for function: $function" | tee -a $file_logs
      local payload="${functions[$function]//<VALUE>/$value}"

      echo "Payload: $value" >>"$file_info"
      echo "Start-time: $(date +%s).$(date +%N)" >>"$file_info"

      for ((i = 1; i <= $runs; i++)); do
        execution_time "$function" "$runtime" "$cold_start" "$payload"
      done

      echo "End-time: $(date +%s).$(date +%N)" >>"$file_info"
      echo "-----" >>"$file_info"

      echo "" >>"$file_time"
      echo "" >>"$file_pod_id"

    done
  else

    echo "No payload" >>"$file_info"
    echo "Start-time: $(date +%s).$(date +%N)" >>"$file_info"

    for ((i = 1; i <= $runs; i++)); do
      execution_time "$function" "$runtime" "$cold_start"
    done

    echo "End-time: $(date +%s).$(date +%N)" >>"$file_info"
    echo "-----" >>"$file_info"

    echo "" >>"$file_time"
    echo "" >>"$file_pod_id"
  fi

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
  echo "Usage: <function> <runtime> <startup_type>"
  echo "Available functions:"
  printf "  all\n"
  printf "  %s\n" "${!functions[@]}"
  echo "Runtime options:"
  echo "  all"
  echo "  native"
  echo "  wasm"
  echo "  wasm-aot"
  echo "Startup options:"
  echo "  all"
  echo "  warm"
  echo "  cold"
  exit 1
fi
function=${1:-"all"}
runtime=${2:-"all"}
startup_type=${3:-"all"}

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
  echo "Removing the execution_time directory and its contents..."
  rm -rf "$execution_time_path"
else
  echo "Directory not removed, as clear_files_on_run is not true."
fi

mkdir -p $root_path

run_test() {
  local func="$1"
  local run="$2"
  local startup_type="$3"

  # Cleanup before run
  #delete_all_active_pods
  delete_all_ksvc

  deploy_ksvc "$func" "$run" "$startup_type"

  echo "Running execution time test for $func with $run runtime - $startup_type" | tee -a $file_logs
  if [ "$startup_type" == "cold" ]; then
    process_execution_time_runs "$func" "$run" "true"
  else
    process_execution_time_runs "$func" "$run"
  fi
}

if [[ "$function" == "all" ]]; then
  # Run tests for all functions
  for func in "${!functions[@]}"; do
    if [[ "$runtime" == "all" ]]; then
      for run in "native" "wasm" "wasm-aot"; do
        # Run both warm and cold tests
        if [[ "$startup_type" == "all" ]]; then
          for startup in "warm" "cold"; do
            run_test "$func" "$run" "$startup"
          done
        else
          run_test "$func" "$run" "$startup_type"
        fi
      done
    else
      if [[ "$startup_type" == "all" ]]; then
        for startup in "warm" "cold"; do
          run_test "$func" "$runtime" "$startup"
        done
      else
        run_test "$func" "$runtime" "$startup_type"
      fi
    fi
  done
else
  if [[ " ${!functions[@]} " =~ " $function " ]]; then
    # Run specific function tests
    if [[ "$runtime" == "all" ]]; then
      for run in "native" "wasm" "wasm-aot"; do
        if [[ "$startup_type" == "all" ]]; then
          for start in "warm" "cold"; do
            run_test "$function" "$run" "$start"
          done
        else
          run_test "$function" "$run" "$startup_type"
        fi
      done
    else
      if [[ "$startup_type" == "all" ]]; then
        for start in "warm" "cold"; do
          run_test "$function" "$runtime" "$start"
        done
      else
        run_test "$function" "$runtime" "$startup_type"
      fi
    fi
  else
    echo "Unknown function: $function"
    exit 1
  fi
fi

echo "Measure script completed." | tee -a $file_logs
