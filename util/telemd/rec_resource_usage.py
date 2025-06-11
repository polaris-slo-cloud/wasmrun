import os
import sys

import redis
import telemc

rds = redis.Redis(host="192.168.1.111", decode_responses=True)

home_directory = os.environ.get('HOME')
path = sys.argv[1]

type = sys.argv[2]

data_mem_filepath = os.path.join(home_directory, path , 'mem.csv')
data_cpu_filepath = os.path.join(home_directory, path , 'cpu.csv')

with open(data_mem_filepath, "w") as mem_file, open(data_cpu_filepath, "w") as cpu_file:
    with telemc.TelemetrySubscriber(rds) as sub:

        for telem in sub:

            data = f"{telem.timestamp},{telem.node},{telem.value},{telem.metric},{telem.subsystem}\n"

            # measure for pods
            if type == "execution_time":
                if telem.metric == "kubernetes_cgrp_memory":
                    mem_file.write(data)
                elif telem.metric == "kubernetes_cgrp_cpu":
                    cpu_file.write(data)
            # measure for entire node
            elif type == "throughput":
                if telem.metric == "ram":
                    mem_file.write(data)
                elif telem.metric == "cpu":
                    cpu_file.write(data)
            else:
                raise ValueError(f"Invalid type '{type}' provided. Expected 'execution_time' or 'throughput'.")
