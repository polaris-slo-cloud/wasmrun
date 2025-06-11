#!/bin/bash
TARGETARCH="aarch64"

export IMAGE_REGISTRY="noah1170"
export AWS_ACCESS_KEY_ID=""
export AWS_SECRET_ACCESS_KEY=""
export AWS_DEFAULT_REGION=""
export REDIS_URL=""

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

build_test_redis_wasm() {
    echo "Building test-redis-wasm..."
    buildah build -f Dockerfile -t test-redis-wasm ./test-redis-wasm

    echo "Pushing test-redis-wasm to $IMAGE_REGISTRY..."
    buildah push test-redis-wasm $IMAGE_REGISTRY/test-redis-wasm:latest
}

deploy_test_redis_wasm() {
    echo "Deploying test-redis-wasm..."

    export FUNCTION_NAME="test-redis-wasm"

    envsubst <"$script_dir/../../util/service-wasm.yaml.template" | microk8s kubectl apply -f -
}

build_test_redis_native() {
    echo "Building test-redis-native..."
    buildah build --build-arg TARGETARCH="$TARGETARCH" -f Dockerfile -t test-redis-native ./test-redis-native

    echo "Pushing test-redis-native to $IMAGE_REGISTRY..."
    buildah push test-redis-native $IMAGE_REGISTRY/test-redis-native:latest
}

deploy_test_redis_native() {
    echo "Deploying test-redis-native..."

    export FUNCTION_NAME="test-redis-native"

    envsubst <"$script_dir/../../util/service-native.yaml.template" | microk8s kubectl apply -f -
}

build_test_redis_native
deploy_test_redis_native

build_test_redis_wasm
deploy_test_redis_wasm