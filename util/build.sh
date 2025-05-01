#!/bin/bash

# load config
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

REPO_ROOT=$(dirname "$(dirname "$(realpath "$0")")")
echo $REPO_ROOT

build-native() {
  echo "Building $1-native..."

  buildah build \
    --build-arg TARGETARCH="$TARGETARCH" \
    --build-arg FUNCTION_PATH="functions/src/$1/native" \
    -f "functions/src/$1/native/Dockerfile" \
    -t "$1-native" \
    "$REPO_ROOT"

  echo "Pushing $1-native to $IMAGE_REGISTRY..."
  buildah push "$1-native" "$IMAGE_REGISTRY/$1-native:latest"
}

build-wasm() {
  echo "Building $1-wasm..."

  buildah build \
  --build-arg FUNCTION_PATH="functions/src/$1/wasm" \
  -f "functions/src/$1/wasm/Dockerfile" \
  --annotation "module.wasm.image/variant=compat-smart" \
  -t "$1-wasm" \
  "$REPO_ROOT"

  echo "Pushing $1-wasm to $IMAGE_REGISTRY..."
  buildah push $1-wasm $IMAGE_REGISTRY/$1-wasm:latest
}

if [ "$#" -lt 1 ]; then
  echo "No build function provided."
  echo "Available functions:"
  for function in "${functions[@]}"; do
    echo "  $function"
  done
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

buildah login $REGISTRY_PLATFORM --username $REGISTRY_USER --password $REGISTRY_PASS

case $function in
all)
  for item in "${functions[@]}"; do
    if [ "$build_type" == "native" ] || [ "$build_type" == "all" ]; then
      build-native $item
    fi
    if [ "$build_type" == "wasm" ] || [ "$build_type" == "all" ]; then
      build-wasm $item
    fi
  done
  ;;
*)
  for item in "${functions[@]}"; do
    if [[ "$item" == "$function" ]]; then
      if [ "$build_type" == "native" ] || [ "$build_type" == "all" ]; then
        build-native $item
      fi
      if [ "$build_type" == "wasm" ] || [ "$build_type" == "all" ]; then
        build-wasm $item
      fi
      exit 0
    fi
  done

  echo "Unknown build function: $function"
  echo "Usage: $0 [all|<function-name>] [native|wasm]"
  exit 1
  ;;
esac

echo "Build script completed."
