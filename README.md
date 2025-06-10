# Benchmarking Wasm-based Serverless Functions

## Functions
| Function | Category | Resource Utilization | Output - Storage Type | Input - Storage Type | Language | Source | Description |
| ----------- | ----------- | ----------- | ----------- | ----------- | ----------- | ----------- | ----------- |
| Hello-World | Basic | - | - | - | Rust | - | [Desc](docs/functions/Hello-World.md) |
| Fibonacci | Computational | CPU | - | - | Rust | [vhive-serverless](https://github.com/vhive-serverless/vSwarm/tree/main/benchmarks/fibonacci) | [Desc](docs/functions/Fibonacci.md) |
| Image-Resize | Multimedia | I/O , Memory| S3, Local, Redis | - | Rust | [spcl/serverless-benchmarks](https://github.com/spcl/serverless-benchmarks/tree/master/benchmarks/200.multimedia/210.thumbnailer/python) | [Desc](docs/functions/Image-Resize.md) |
| Audio-Generation | Multimedia | I/O , CPU | S3, Local, Redis | - | Rust | [korvoj/wasm-serverless-benchmarks](https://github.com/korvoj/wasm-serverless-benchmarks/tree/master/functions/rust/audio-sine-wave) | [Desc](docs/functions/Audio-Generation.md) |
| Fuzzy-Search | - | I/O , CPU | - | S3, Local, Redis | Rust | [korvoj/wasm-serverless-benchmarks](https://github.com/korvoj/wasm-serverless-benchmarks/tree/master/functions/rust/fuzzysearch) | [Desc](docs/functions/Fuzzy-Search.md) |
| Get-Prime-Numbers | Computational | CPU | - | -  | Rust | [korvoj/wasm-serverless-benchmarks](https://github.com/korvoj/wasm-serverless-benchmarks/tree/master/functions/rust/prime-numbers) | [Desc](docs/functions/Get-Prime-Numbers.md) |
| Language-Detection | - | I/O , CPU , Memory | - | S3, Local, Redis  | Rust | [korvoj/wasm-serverless-benchmarks](https://github.com/korvoj/wasm-serverless-benchmarks/tree/master/functions/rust/whatlang) | [Desc](docs/functions/Language-Detection.md) |
| Planet-System-Simulation | Scientific | CPU , Memory | - | -  | Rust | [korvoj/wasm-serverless-benchmarks](https://github.com/korvoj/wasm-serverless-benchmarks/tree/master/functions/rust/n-body) | [Desc](docs/functions/Planet-System-Simulation.md) |
| Encrypt-Message | Security |  I/O , CPU | - | S3, Local, Redis | Rust | [vhive-serverless](https://github.com/vhive-serverless/vSwarm/tree/main/benchmarks/aes) | [Desc](docs/functions/Encrypt-Message.md) |
| Decrypt-Message | Security |  I/O , CPU | - | S3, Local, Redis | Rust | [backendengineer](https://backendengineer.io/aes-encryption-rust) | [Desc](docs/functions/Decrypt-Message.md) |
| Create-Mandelbrot-Bitmap | - | I/O , CPU | S3, Local, Redis | - | Rust | [BenchmarksGame](https://benchmarksgame-team.pages.debian.net/benchmarksgame/program/mandelbrot-rust-4.html) | [Desc](docs/functions/Create-Mandelbrot-Bitmap.md) |
| Template | - | - | - | - | Rust | - | [Desc](docs/functions/Template.md) |


## Measurement
The following part explains the setup needed to reproduce the measurement results for execution times and scaleability in the `$HOME/telemetry` folder.

---
### Prerequisites

- The steps to create the inital setup are explained in [SETUP.md](docs/SETUP.md)
- Many of the serverless functions use input files which are located in the [Resources](resources/files) folder. These files must be saved in Redis (or locally if this storage solution is prefered) under the their filename (including the file extension) e.g. with redis-cli: `redis-cli -x set search_text_1kb.txt`.

```
for file in resources/files/*; do redis-cli -x set "files:$(basename "$file")" < "$file"; done
```

---

### 1. Config

The [config.sh](util/config.sh) bash script contains global configuration variables which are mandatory for the subsequent building and deployment of the functions.

The following parameters must be set in the script:
- `REGISTRY_PLATFORM` (e.g. docker.io, ghcr.io, .. )
- `IMAGE_REGISTRY`
- `REGISTRY_USER`
- `REGISTRY_PASS`
- `TARGETARCH` (e.g. aarch64, x86_64, .. )
- `GATEWAY_URL` (if no other networking layer was configured the default gateway should be kourier-internal)

These are optional depending on the storage solution prefered:
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_DEFAULT_REGION`
- `REDIS_URL`

---

### 2. Building the functions

With the help of the [build.sh](util/build.sh) bash script all functions can be built. This step is mandatory.

---

### 2. Deploying the functions

With the help of the [deploy.sh](util/deploy.sh) bash script all functions can be deplyoed individually but this is not necessary because both measurement scripts source this file and deploy the functions as needed.

---

### 3. Running the experiments

The framework provides 2 types of expirements:

- Execution time: Designed to measure the execution time and resource usage of various serverless functions in a Kubernetes environment. It supports measurement for both **cold** and **warm starts**.

- Scaleability: Designed to measure the scalability/throughput and resource usage of various serverless functions under different levels of concurrency.

Measurement data is saved to `~/telemetry`. 

The following paramets can be set in the scripts:
- clear_files_on_run:
A boolean flag to check if measurement data from previous recordings should be deleted from the `~/telemetry` directory.
- runs:
Number of runs to test against, intended to eliminate bias.





#### 3.1 Execution time

The script [measure_execution_time.sh](util/measure_execution_time.sh) is intended to run the experiments with a default value of 5 runs per function and payload. The payload values and runs can be configured in the script as desired.

#### 3.2 Scaleability

The script [measure_scaleability.sh](util/measure_scaleability.sh) is intended  run the experiments for concurrency levels of (10 20 30 40 50 60 70 80 100 200) and a number of 5 runs. This can be configured in the script.

---

### 4. Filtering of resource usage data

The script [filter_resource_usage_data.py](measurement_data/filter_resource_usage_data.py) helps splitting the resource usage files for the performed runs and also trims the data.

---

### Note

- If some functions return an activator request timeout it is recommened to increase the value of `timeoutSeconds` in the corresponding Knative Service [service-native-cold.yaml.template](util/service-native-cold.yaml.template) , [service-wasm-cold.yaml.template](util/service-wasm-cold.yaml.template).
