#!/bin/bash
IMAGE_REGISTRY="noah1170"
TARGETARCH="aarch64"

build_check_connection_wasm() {
  cd test-functions/check-connection/wasm

  echo "Building check-connection-wasm..."
  buildah build -f Dockerfile -t check-connection-wasm .

  echo "Pushing check-connection-wasm to $IMAGE_REGISTRY..."
  buildah push check-connection-wasm $IMAGE_REGISTRY/check-connection-wasm:latest

  cd - || exit
}

build_communicate_container_wasm() {
  cd test-functions/communicate-container/wasm

  cd server

  echo "Building communicate-container-wasm-server..."
  buildah build -f Dockerfile -t communicate-container-server-wasm --annotation "module.wasm.image/variant=compat-smart" .

  echo "Pushing communicate-container-server-wasm to $IMAGE_REGISTRY..."
  buildah push communicate-container-server-wasm $IMAGE_REGISTRY/communicate-container-server-wasm:latest

  cd ../client

  echo "Building communicate-container-wasm-client..."
  buildah build -f Dockerfile -t communicate-container-client-wasm --annotation "module.wasm.image/variant=compat-smart" .

  echo "Pushing communicate-container-client-wasm to $IMAGE_REGISTRY..."
  buildah push communicate-container-client-wasm $IMAGE_REGISTRY/communicate-container-client-wasm:latest

  cd - || exit
}

build_test_http_connection_native() {
  cd test-functions/test-http-connection/native

  echo "Building test-http-connection-native..."
  buildah build -f Dockerfile -t test-http-connection-native --build-arg TARGETARCH="$TARGETARCH" .

  echo "Pushing test-http-connection-native to $IMAGE_REGISTRY..."
  buildah push test-http-connection-native $IMAGE_REGISTRY/test-http-connection-native:latest

  cd - || exit
}

build_test_http_connection_wasm() {
  cd test-functions/test-http-connection/wasm

  echo "Building test-http-connection-wasm..."
  buildah build -f Dockerfile -t test-http-connection-wasm --annotation "module.wasm.image/variant=compat-smart" .

  echo "Pushing test-http-connection-wasm to $IMAGE_REGISTRY..."
  buildah push test-http-connection-wasm $IMAGE_REGISTRY/test-http-connection-wasm:latest

  cd - || exit
}

if [ "$#" -lt 1 ]; then
  echo "No build function provided."
  read -p "Please enter the build function <function-name> (default: all): " function
  function=${function:-all}
  read -p "Please enter the build type [native|wasm] (default: all): " build_type
  build_type=${build_type:-all}
elif [ "$#" -eq 1 ]; then
  function=$1
  build_type="all"
elif [ "$#" -eq 2 ]; then
  function=$1
  build_type=$2
else
  echo "Too many arguments."
  echo "Usage: $0 [<function-name>] [native|wasm]"
  exit 1
fi

# Process the provided function
case $function in
test-http-connection)
  if [ "$build_type" == "wasm" ] || [ "$build_type" == "all" ]; then
    build_test_http_connection_wasm
  elif [ "$build_type" == "native" ] || [ "$build_type" == "all" ]; then
    build_test_http_connection_native
  fi
  ;;
check-connection)
  if [ "$build_type" == "wasm" ] || [ "$build_type" == "all" ]; then
    build_check_connection-wasm
  fi
  ;;
communicate-container)
  if [ "$build_type" == "wasm" ] || [ "$build_type" == "all" ]; then
    build_communicate_container_wasm
  fi
  ;;
all)
  if [ "$build_type" == "wasm" ] || [ "$build_type" == "all" ]; then
    check_connection_wasm
    build_communicate_container_wasm
  fi
  ;;
*)
  echo "Unknown build function: $function"
  echo "Usage: $0 [all|<function-name>] [native|wasm]"
  exit 1
  ;;
esac

echo "Build script completed."
