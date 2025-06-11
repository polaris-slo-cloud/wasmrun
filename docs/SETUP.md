# Setup

## Prerequisites

- Buildah

`sudo apt install buildah -y`


## Microk8s 

### 1. Install microk8s

`sudo snap install microk8s --channel=1.30/stable --classic`

### 2. Add current user the 'microk8s' group 
```
sudo usermod -a -G microk8s $USER
newgrp microk8s
```
### 3. Check if installation finished
`microk8s status --wait-ready`

### 4 .Create kube config
```
microk8s kubectl version
sudo chown -R $USER ~/.kube
microk8s config > ~/.kube/config
```

### 5. Create alias for microk8s kubectl (Optional)
```
echo "alias kubectl='microk8s kubectl'" >> ~/.bashrc
source ~/.bashrc
```

### 6. Enable core dns addon
`microk8s enable dns`

### 7. Enable community addons
```
git config --global --add safe.directory /snap/microk8s/current/addons/community/.git
microk8s enable community
```

## Add Nodes to cluster (Optional)

### 1. Get token (Master)
`microk8s add-node`

### 2. Join Node (Worker)
`microk8s join <master-ip>:25000/<token> --worker`

### 3. Verify joined nodes
`microk8s kubectl get nodes`


## Hostpath storage

### 1. Enable hostpath storage addon
`microk8s enable hostpath-storage`

### 2. Add pvc
`kubectl apply -f ~/bachelor/util/hostpath-storage/pvc.yaml`


## WASM

### 1. Enable kwasm addon
`microk8s enable kwasm`

### 2. Deploy runtime

`kubectl apply -f ~/bachelor/util/wasmedge-runtime.yaml`

## Knative

### 1. Enable knative addon
`microk8s enable knative`

### 2. Enable runtimeClass Feature Flag
```
kubectl patch --namespace knative-serving configmap/config-features \
 --type merge \
 --patch '{"data":{"kubernetes.podspec-runtimeclassname": "enabled"}}'
```

### 3. Enable PVCs for service
```
kubectl patch --namespace knative-serving configmap/config-features \
 --type merge \
 --patch '{"data":{"kubernetes.podspec-persistent-volume-claim": "enabled", "kubernetes.podspec-persistent-volume-write": "enabled"}}'
```

### 4. Set domain for knative serving (Optional)
```
microk8s kubectl patch configmap/config-domain \
--namespace knative-serving \
--type merge \
--patch '{"data":{"example.com":""}}'
```

## Benchmarking Utilities

### 1. Redis

#### 1.1 Add repo
```
sudo apt install redis-tools

sudo snap install redis

redis-cli CONFIG SET protected-mode no
```


### 2. Telemd

#### 2.1 Build telemd
```
git clone https://github.com/edgerun/telemd.git
cd telemd
makedocker
docker tag edgerun/telemd:latest <your-dockerhub-username>/telemd:latest
docker push <your-dockerhub-username>/telemd:latest
```

#### 2.2 Apply configMap
`kubectl apply -f ~/bachelor/util/telemd/telemd-configMap.yaml`

#### 2.3 Deploy telemd daemonset

```
kubectl create namespace telemd
kubectl apply -f ~/bachelor/util/telemd/telemd-daemonSet.yaml
```

## Functions

### Prerequisites

### Buildah
```
sudo apt-get -y update
sudo apt-get -y install buildah
```


