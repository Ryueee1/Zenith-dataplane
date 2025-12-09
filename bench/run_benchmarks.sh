#!/bin/bash
# =============================================================================
# Zenith Benchmark Runner
# Author: Wahyu Ardiansyah
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Default configuration
NUM_WORKERS=${ZENITH_NUM_WORKERS:-4}
BATCH_SIZE=${ZENITH_BATCH_SIZE:-32}
DURATION=${ZENITH_DURATION:-60}
DATA_DIR="${ZENITH_BENCHMARK_DATA:-$SCRIPT_DIR/data}"
RESULTS_DIR="${ZENITH_BENCHMARK_RESULTS:-$SCRIPT_DIR/results}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Timestamp for this run
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RUN_DIR="$RESULTS_DIR/$TIMESTAMP"

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --workers N      Number of data loading workers (default: $NUM_WORKERS)"
    echo "  --batch-size N   Batch size (default: $BATCH_SIZE)"
    echo "  --duration N     Test duration in seconds (default: $DURATION)"
    echo "  --all            Run all benchmarks"
    echo "  --pytorch        Run PyTorch baseline only"
    echo "  --zenith         Run Zenith benchmark only"
    echo "  --help           Show this help message"
    echo ""
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Parse arguments
RUN_ALL=false
RUN_PYTORCH=false
RUN_ZENITH=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --workers)
            NUM_WORKERS="$2"
            shift 2
            ;;
        --batch-size)
            BATCH_SIZE="$2"
            shift 2
            ;;
        --duration)
            DURATION="$2"
            shift 2
            ;;
        --all)
            RUN_ALL=true
            shift
            ;;
        --pytorch)
            RUN_PYTORCH=true
            shift
            ;;
        --zenith)
            RUN_ZENITH=true
            shift
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Default to all if nothing specified
if [[ "$RUN_ALL" == "false" && "$RUN_PYTORCH" == "false" && "$RUN_ZENITH" == "false" ]]; then
    RUN_ALL=true
fi

# Setup
echo ""
echo "=============================================="
echo "Zenith Benchmark Suite"
echo "=============================================="
echo "Timestamp: $TIMESTAMP"
echo "Workers: $NUM_WORKERS"
echo "Batch Size: $BATCH_SIZE"
echo "Duration: ${DURATION}s"
echo "Data Dir: $DATA_DIR"
echo "Results Dir: $RUN_DIR"
echo "=============================================="
echo ""

# Create results directory
mkdir -p "$RUN_DIR"

# Activate virtual environment
if [ -f "$PROJECT_ROOT/.venv/bin/activate" ]; then
    source "$PROJECT_ROOT/.venv/bin/activate"
    log_info "Activated virtual environment"
fi

# Export configuration
export ZENITH_NUM_WORKERS=$NUM_WORKERS
export ZENITH_BATCH_SIZE=$BATCH_SIZE
export ZENITH_BENCHMARK_DATA=$DATA_DIR
export ZENITH_CORE_LIB="$PROJECT_ROOT/target/release/libzenith_core.so"
export PYTHONPATH="$PROJECT_ROOT/sdk-python:$PYTHONPATH"

# Check for datasets
if [ ! -d "$DATA_DIR" ] || [ -z "$(ls -A $DATA_DIR 2>/dev/null)" ]; then
    log_warn "No datasets found. Generating small benchmark datasets..."
    python "$SCRIPT_DIR/generate_datasets.py" --scale small --output-dir "$DATA_DIR"
fi

# Save configuration
cat > "$RUN_DIR/config.json" << EOF
{
    "timestamp": "$TIMESTAMP",
    "num_workers": $NUM_WORKERS,
    "batch_size": $BATCH_SIZE,
    "duration_seconds": $DURATION,
    "data_dir": "$DATA_DIR",
    "hostname": "$(hostname)",
    "python_version": "$(python --version 2>&1)"
}
EOF

# Run benchmarks
run_benchmark() {
    local name=$1
    local script=$2
    
    log_info "Running $name benchmark..."
    
    if [ -f "$script" ]; then
        python "$script" \
            --duration "$DURATION" \
            --workers "$NUM_WORKERS" \
            --batch-size "$BATCH_SIZE" \
            --output "$RUN_DIR/${name}_results.json" \
            2>&1 | tee "$RUN_DIR/${name}_log.txt"
        
        if [ $? -eq 0 ]; then
            log_success "$name benchmark completed"
        else
            log_error "$name benchmark failed"
        fi
    else
        log_warn "Script not found: $script"
    fi
}

# PyTorch Baseline
if [[ "$RUN_ALL" == "true" || "$RUN_PYTORCH" == "true" ]]; then
    run_benchmark "pytorch" "$SCRIPT_DIR/baselines/pytorch_baseline.py"
fi

# Zenith
if [[ "$RUN_ALL" == "true" || "$RUN_ZENITH" == "true" ]]; then
    run_benchmark "zenith" "$SCRIPT_DIR/zenith/zenith_benchmark.py"
fi

# Generate summary report
log_info "Generating summary report..."

python << EOF
import json
import os
from pathlib import Path

run_dir = Path("$RUN_DIR")
results = {}

# Collect all results
for result_file in run_dir.glob("*_results.json"):
    name = result_file.stem.replace("_results", "")
    try:
        with open(result_file) as f:
            results[name] = json.load(f)
    except:
        pass

if not results:
    print("No results found")
    exit(0)

# Generate markdown report
report = f"""# Zenith Benchmark Report

**Timestamp:** $TIMESTAMP
**Configuration:** Workers={$NUM_WORKERS}, BatchSize={$BATCH_SIZE}, Duration={$DURATION}s

## Results Summary

| Loader | Throughput (samples/s) | Latency p50 (ms) | Latency p99 (ms) |
|--------|------------------------|------------------|------------------|
"""

for name, data in results.items():
    throughput = data.get("throughput", 0)
    p50 = data.get("latency_p50_ms", 0)
    p99 = data.get("latency_p99_ms", 0)
    report += f"| {name} | {throughput:,.0f} | {p50:.2f} | {p99:.2f} |\n"

report += """
## Environment

See \`config.json\` and \`environment.txt\` for full details.

---
*Generated by Zenith Benchmark Suite*
"""

with open(run_dir / "benchmark_report.md", "w") as f:
    f.write(report)

print(f"Report saved to: {run_dir / 'benchmark_report.md'}")
EOF

echo ""
echo "=============================================="
log_success "Benchmark run complete!"
echo "Results: $RUN_DIR"
echo "=============================================="
echo ""
