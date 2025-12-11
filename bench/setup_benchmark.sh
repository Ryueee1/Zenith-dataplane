#!/bin/bash
# =============================================================================
# Zenith Benchmark Environment Setup
# Author: Wahyu Ardiansyah
# =============================================================================

set -e

echo "=============================================="
echo "Zenith Benchmark Environment Setup"
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check Python version
echo -e "\n${YELLOW}[1/6] Checking Python version...${NC}"
python_version=$(python3 --version 2>&1 | cut -d' ' -f2)
echo "Python version: $python_version"

# Create virtual environment if not exists
echo -e "\n${YELLOW}[2/6] Setting up virtual environment...${NC}"
if [ ! -d ".venv" ]; then
    python3 -m venv .venv
    echo "Created new virtual environment"
else
    echo "Using existing virtual environment"
fi

source .venv/bin/activate

# Install dependencies
echo -e "\n${YELLOW}[3/6] Installing Python dependencies...${NC}"
pip install --upgrade pip > /dev/null
pip install -q \
    torch \
    torchvision \
    pyarrow \
    pandas \
    numpy \
    pyyaml \
    tqdm \
    psutil \
    webdataset \
    matplotlib \
    tabulate

# Check for NVIDIA GPU and install DALI
echo -e "\n${YELLOW}[4/6] Checking GPU and DALI...${NC}"
if command -v nvidia-smi &> /dev/null; then
    echo "NVIDIA GPU detected"
    nvidia-smi --query-gpu=name,memory.total --format=csv,noheader
    
    # Install DALI
    pip install -q nvidia-dali-cuda120 2>/dev/null || \
    pip install -q nvidia-dali-cuda118 2>/dev/null || \
    echo "DALI installation skipped (CUDA version not supported)"
else
    echo "No NVIDIA GPU detected - DALI benchmarks will be skipped"
fi

# Create directory structure
echo -e "\n${YELLOW}[5/6] Creating directory structure...${NC}"
mkdir -p bench/configs
mkdir -p bench/baselines
mkdir -p bench/zenith
mkdir -p bench/reports
mkdir -p bench/results
mkdir -p bench/data

# Record environment info
echo -e "\n${YELLOW}[6/6] Recording environment information...${NC}"
cat > bench/results/environment.txt << EOF
============================================
Zenith Benchmark Environment
Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
============================================

SYSTEM:
- Hostname: $(hostname)
- OS: $(uname -s) $(uname -r)
- Architecture: $(uname -m)

CPU:
$(lscpu | grep -E "Model name|CPU\(s\)|Thread|Core" | head -5)

MEMORY:
$(free -h | grep Mem)

STORAGE:
$(df -h . | tail -1)

PYTHON:
- Version: $python_version
- PyTorch: $(python3 -c "import torch; print(torch.__version__)" 2>/dev/null || echo "not installed")
- PyArrow: $(python3 -c "import pyarrow; print(pyarrow.__version__)" 2>/dev/null || echo "not installed")

GPU:
$(nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader 2>/dev/null || echo "No GPU detected")

CUDA:
$(nvcc --version 2>/dev/null | grep release || echo "CUDA not installed")
EOF

echo -e "\n${GREEN}=============================================="
echo "Setup complete!"
echo "=============================================="
echo -e "${NC}"
echo "Environment recorded in: bench/results/environment.txt"
echo ""
echo "Next steps:"
echo "  1. Generate datasets:  python bench/generate_datasets.py"
echo "  2. Run benchmarks:     ./bench/run_benchmarks.sh"
echo ""
