#!/bin/bash
#
# Performance Regression Detection Script
# 
# This script compares current benchmark performance against a saved baseline
# and fails if any benchmark regresses by more than 10%.
#
# Usage:
#   ./scripts/check_performance.sh [baseline_name]
#
# baseline_name: Name of the baseline to compare against (default: "main")
#
# Exit codes:
#   0 = No regressions detected
#   1 = Regression detected or benchmark failed

set -e

# Configuration
BASELINE="${1:-main}"
THRESHOLD=10.0  # Maximum acceptable regression percentage
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Benchmark suites to check
BENCHMARKS=(
    "bitmap_bench"
    "decoder_bench"
    "full_bench"
)

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}  Performance Regression Detection${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""
echo "Baseline: $BASELINE"
echo "Threshold: ${THRESHOLD}% regression"
echo ""

# Check if baseline exists
BASELINE_DIR="target/criterion"
if [ ! -d "$BASELINE_DIR" ]; then
    echo -e "${RED}Error: No criterion directory found.${NC}"
    echo "Run 'cargo bench' first to generate benchmarks."
    exit 1
fi

# Function to run benchmark and capture output
run_benchmark() {
    local bench_name=$1
    local output_file="$TEMP_DIR/${bench_name}.txt"
    
    echo -e "${BLUE}Running $bench_name...${NC}"
    
    # Run benchmark with baseline comparison
    if cargo bench --bench "$bench_name" -- --baseline "$BASELINE" > "$output_file" 2>&1; then
        echo -e "${GREEN}✓${NC} $bench_name completed"
        return 0
    else
        # Check if error is due to missing baseline
        # Criterion error: "Baseline 'main' must exist before comparison is allowed"
        if grep -q "must exist before comparison" "$output_file"; then
            echo -e "${YELLOW}⚠${NC} Baseline '$BASELINE' not found for $bench_name"
            echo -e "  Run: ${YELLOW}cargo bench -- --save-baseline $BASELINE${NC}"
            return 2
        else
            echo -e "${RED}✗${NC} $bench_name failed"
            cat "$output_file"
            return 1
        fi
    fi
}

# Function to check for regressions in benchmark output
check_regressions() {
    local bench_name=$1
    local output_file="$TEMP_DIR/${bench_name}.txt"
    local found_regression=0
    
    # Parse criterion output looking for "change:" lines with regression
    # Format: "change: [+8.1234% +10.5678% +12.9012%]" (regression is positive)
    # We want to extract the middle value (median estimate)
    
    while IFS= read -r line; do
        if echo "$line" | grep -q "change:"; then
            # Extract the percentage change (middle value in brackets)
            # Example: "change: [-0.8234% +0.3421% +1.5438%]"
            change=$(echo "$line" | sed -n 's/.*change:.*\[\([^]]*\)\].*/\1/p')
            
            if [ -n "$change" ]; then
                # Extract middle value (median)
                median=$(echo "$change" | awk '{print $2}' | tr -d '%+')
                
                # Check if regression (positive change) exceeds threshold
                # Use bc for floating point comparison
                if (( $(echo "$median > $THRESHOLD" | bc -l) )); then
                    # Get the benchmark name from previous lines
                    bench_test=$(grep -B 2 "change:" "$output_file" | grep -v "change:" | grep -v "time:" | tail -1 | awk '{print $1}')
                    echo -e "${RED}  ✗ REGRESSION:${NC} $bench_test regressed by ${RED}${median}%${NC} (threshold: ${THRESHOLD}%)"
                    found_regression=1
                elif (( $(echo "$median < -$THRESHOLD" | bc -l) )); then
                    # Significant improvement
                    bench_test=$(grep -B 2 "change:" "$output_file" | grep -v "change:" | grep -v "time:" | tail -1 | awk '{print $1}')
                    improvement=$(echo "$median" | tr -d '-')
                    echo -e "${GREEN}  ✓ IMPROVEMENT:${NC} $bench_test improved by ${GREEN}${improvement}%${NC}"
                fi
            fi
        fi
    done < "$output_file"
    
    return $found_regression
}

# Main execution
echo -e "${BLUE}Running benchmarks...${NC}"
echo ""

OVERALL_STATUS=0
MISSING_BASELINE=0

for bench in "${BENCHMARKS[@]}"; do
    if run_benchmark "$bench"; then
        check_regressions "$bench" || OVERALL_STATUS=1
    else
        exit_code=$?
        if [ $exit_code -eq 2 ]; then
            MISSING_BASELINE=1
        else
            OVERALL_STATUS=1
        fi
    fi
    echo ""
done

# Final summary
echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}  Summary${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""

if [ $MISSING_BASELINE -eq 1 ]; then
    echo -e "${YELLOW}⚠ Missing baseline '$BASELINE'${NC}"
    echo -e "  Create baseline: ${YELLOW}cargo bench -- --save-baseline $BASELINE${NC}"
    echo ""
    echo -e "${YELLOW}Note: This is not considered a failure.${NC}"
    echo -e "      Establish the baseline, then re-run this script."
    exit 0
elif [ $OVERALL_STATUS -eq 0 ]; then
    echo -e "${GREEN}✓ All benchmarks passed!${NC}"
    echo -e "  No regressions detected (threshold: ${THRESHOLD}%)"
    exit 0
else
    echo -e "${RED}✗ Regression detected!${NC}"
    echo -e "  One or more benchmarks regressed by more than ${THRESHOLD}%"
    echo ""
    echo "Review the regression and consider:"
    echo "  1. Is the regression acceptable for this change?"
    echo "  2. Can the code be optimized to avoid the regression?"
    echo "  3. Should the baseline be updated if this is intentional?"
    exit 1
fi
