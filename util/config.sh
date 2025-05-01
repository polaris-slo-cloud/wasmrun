#!/bin/bash

# Configuration

## use microk8s kubectl by default
alias kubectl='microk8s kubectl'

## registry
REGISTRY_PLATFORM="docker.io"
IMAGE_REGISTRY=""
REGISTRY_USER=""
REGISTRY_PASS=""

## platform
TARGETARCH="aarch64"

## knative
GATEWAY_URL="http://10.152.183.182"

## storage solutions
AWS_ACCESS_KEY_ID=""
AWS_SECRET_ACCESS_KEY=""
AWS_DEFAULT_REGION=""

REDIS_URL="redis://192.168.1.10/"


# export for service yaml files
export REGISTRY_PLATFORM IMAGE_REGISTRY REGISTRY_USER REGISTRY_PASS REDIS_URL AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY AWS_DEFAULT_REGION