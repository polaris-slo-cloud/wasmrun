#!/bin/bash

# Configuration
source config.sh

functions=(
  "hello-world"
  "fibonacci"
  "audio-generation"
  "image-resize"
  "fuzzy-search"
  "get-prime-numbers"
  "language-detection"
  "planet-system-simulation"
  "encrypt-message"
  "decrypt-message"
  "create-mandelbrot-bitmap"
)

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

deploy_function_native() {
  local function_name=$1
  local startup_type=$2

  export FUNCTION_NAME="${function_name}-native"

  echo "Deploying ${FUNCTION_NAME}..."

  if [ "$startup_type" == "cold" ]; then
    envsubst < "$script_dir/service-native-cold.yaml.template" | kubectl apply -f -
  else
    envsubst < "$script_dir/service-native.yaml.template" | kubectl apply -f -
  fi
}

# Deploy wasm function
deploy_function_wasm() {
  local function_name=$1
  export FUNCTION_NAME="${function_name}-wasm"

  echo "Deploying ${FUNCTION_NAME}..."

  if [ "$startup_type" == "cold" ]; then
    envsubst < "$script_dir/service-wasm-cold.yaml.template" | kubectl apply -f -
  else
    envsubst < "$script_dir/service-wasm.yaml.template" | kubectl apply -f -
  fi
}

if [ "$#" -eq 0 ]; then
  echo "No deployment target provided."
  echo "Available functions:"
  for function in "${functions[@]}"; do
    echo "  $function"
  done
  read -p "Please enter the deployment target <function-name> (default: all): " function
  function=${function:-all}
  read -p "Please enter the deployment type [native|wasm] (default: all): " deployment_type
  deployment_type=${deployment_type:-all}
  read -p "Please enter the startup type to measure [cold|warm] (default: warm): " startup_type
  s=${startup_type:-warm}
elif [ "$#" -eq 1 ]; then
  function=$1
  deployment_type="all"
  startup_type="warm"
elif [ "$#" -eq 2 ]; then
  function=$1
  deployment_type=$2
  startup_type="warm"
elif [ "$#" -eq 3 ]; then
  function=$1
  deployment_type=$2  
  startup_type=$3
else
  echo "Too many arguments."
  echo "Usage: $0 [<function-name>] [native|wasm] [cold|warm]"
  exit 1
fi

buildah login $REGISTRY_PLATFORM --username $REGISTRY_USER --password $REGISTRY_PASS

# Process deployment
case $function in
  all)
    for func in "${functions[@]}"; do
      if [ "$deployment_type" == "native" ] || [ "$deployment_type" == "all" ]; then
        deploy_function_native "$func" "$startup_type"
      fi
      if [ "$deployment_type" == "wasm" ] || [ "$deployment_type" == "all" ]; then
        deploy_function_wasm "$func" "$startup_type"
      fi
    done
    ;;
  *)
    if [[ " ${functions[*]} " == *" $function "* ]]; then
      if [ "$deployment_type" == "native" ] || [ "$deployment_type" == "all" ]; then
        deploy_function_native "$function" "$startup_type"
      fi
      if [ "$deployment_type" == "wasm" ] || [ "$deployment_type" == "all" ]; then
        deploy_function_wasm "$function" "$startup_type"
      fi
    else
      echo "Unknown deployment target: $function"
      echo "Usage: $0 [all|<function-name>] [native|wasm]"
      exit 1
    fi
    ;;
esac

echo "Deployment script completed."
