#!/bin/bash
#SBATCH --job-name=bench_fdedup
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=6
#SBATCH --mem=32G
#SBATCH --time=48:00:00
#SBATCH --partition=cpu-dedicated
#SBATCH --account=dedicated-cpu@cirad-normal

pixi run cargo build --release --bin benchmark/bin
pixi run ./benchmark.sh
pixi run python plotit.py