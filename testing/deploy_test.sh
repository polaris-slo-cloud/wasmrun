#!/bin/bash
export IMAGE_REGISTRY=noah1170

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

deploy_check_connection_wasm() {
  cd test-functions/check-connection/wasm
  echo "Deploying check-connection-wasm..."
  envsubst < "service-wasm.yaml.template" | microk8s kubectl apply -f -
  cd - || exit
}

deploy_communicate_container_wasm() {
  cd test-functions/communicate-container/wasm/server
  echo "Deploying communicate-container-server-wasm..."
  envsubst < "service-wasm.yaml.template" | microk8s kubectl apply -f -

  cd ../client

  echo "Deploying communicate-container-client-wasm..."
  envsubst < "service-wasm.yaml.template" | microk8s kubectl apply -f -
  cd - || exit
}

deploy_test_http_connection_wasm() {
  cd test-functions/test-http-connection/wasm
  echo "Deploying test-http-connection-wasm..."
  envsubst < "service-wasm.yaml.template" | microk8s kubectl apply -f -
  cd - || exit
}

deploy_test_http_connection_native() {
  cd test-functions/test-http-connection/native
  echo "Deploying test-http-connection-native..."
  envsubst < "service-native.yaml.template" | microk8s kubectl apply -f -
  cd - || exit
}

# Check command line arguments
if [ "$#" -eq 0 ]; then
  echo "No deployment target provided."
  read -p "Please enter the deployment target <function-name> (default: all): " function
  function=${function:-all}
  read -p "Please enter the deployment type [native|wasm] (default: all): " deployment_type
  deployment_type=${deployment_type:-all}
elif [ "$#" -eq 1 ]; then
  function=$1
  deployment_type="all"
elif [ "$#" -eq 2 ]; then
  function=$1
  deployment_type=$2
else
  echo "Too many arguments."
  echo "Usage: $0 [<function-name>] [native|wasm]"
  exit 1
fi

# Process the provided target
case $function in
test-http-connection)
  if [ "$deployment_type" == "wasm" ] || [ "$deployment_type" == "all" ]; then
    deploy_test_http_connection_wasm
  elif [ "$deployment_type" == "native" ] || [ "$deployment_type" == "all" ]; then
    deploy_test_http_connection_native
  fi
  ;;
check-connection)
  if [ "$deployment_type" == "wasm" ] || [ "$deployment_type" == "all" ]; then
    deploy_check_connection_wasm
  fi
  ;;
communicate-container)
  if [ "$deployment_type" == "wasm" ] || [ "$deployment_type" == "all" ]; then
    deploy_communicate_container_wasm
  fi
  ;;
all)
  if [ "$deployment_type" == "wasm" ] || [ "$deployment_type" == "all" ]; then
    deploy_check_connection_wasm
    deploy_communicate_container_wasm
  fi
  ;;
*)
  echo "Unknown deployment target: $function"
  echo "Usage: $0 [all|<function-name>] [native|wasm]"
  exit 1
  ;;
esac

echo "Deployment script completed."
