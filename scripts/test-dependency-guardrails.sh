#!/bin/bash

# Test script to validate dependency guardrails locally
# This script simulates the CI checks to ensure they work before committing

set -e

echo "=== Testing Dependency Management Guardrails ==="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
        exit 1
    fi
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

echo "1. Testing hybrid dependency policy validation..."
# This simulates the dependency-policy job
SECURITY_CRITICAL=(
    "reqwest = \"0.12.26\""
    "tokio = \"1.48.0\""
    "serde = \"1.0.228\""
    "serde_json = \"1.0.145\""
    "blake3 = \"1.8.2\""
)

CORE_DEPS=(
    "clap = \"4.5\""
    "anyhow = \"1.0\""
    "thiserror = \"2.0\""
    "tracing = \"0.1\""
)

POLICY_PASSED=true

# Check security-critical deps
for dep_spec in "${SECURITY_CRITICAL[@]}"; do
    dep_name=$(echo "$dep_spec" | cut -d' ' -f1)
    dep_version=$(echo "$dep_spec" | cut -d'"' -f2)
    
    if grep -q "^$dep_name = \"$dep_version\"" Cargo.toml; then
        echo "  ✅ $dep_name: exact version $dep_version"
    else
        echo "  ❌ $dep_name: expected exact version $dep_version"
        POLICY_PASSED=false
    fi
done

# Check core deps
for dep_spec in "${CORE_DEPS[@]}"; do
    dep_name=$(echo "$dep_spec" | cut -d' ' -f1)
    dep_version=$(echo "$dep_spec" | cut -d'"' -f2)
    
    if grep -q "^$dep_name = \"$dep_version\"" Cargo.toml; then
        echo "  ✅ $dep_name: coarse minimum $dep_version"
    else
        echo "  ❌ $dep_name: expected coarse minimum $dep_version"
        POLICY_PASSED=false
    fi
done

if [ "$POLICY_PASSED" = true ]; then
    print_status 0 "Hybrid dependency policy validation"
else
    print_status 1 "Hybrid dependency policy validation"
fi

echo ""
echo "2. Testing locked dependency resolution..."
if cargo test --lib --bins --locked; then
    print_status 0 "Locked dependency testing"
else
    print_status 1 "Locked dependency testing"
fi

echo ""
echo "3. Testing fresh dependency resolution..."
if cargo update && cargo test --lib --bins; then
    print_status 0 "Fresh resolve testing"
else
    print_status 1 "Fresh resolve testing"
fi

echo ""
echo "4. Testing dependency duplication checks..."
if cargo tree -d > /tmp/dep-tree.txt; then
    DUPLICATE_COUNT=$(grep -c "^[a-zA-Z]" /tmp/dep-tree.txt || echo "0")
    echo "  Found $DUPLICATE_COUNT duplicate dependency groups"
    
    # Check for concerning patterns
    WINDOWS_DUPES=$(grep -i "winapi\|windows\|kernel32\|user32" /tmp/dep-tree.txt | wc -l || echo "0")
    SERDE_DUPES=$(grep -i "serde" /tmp/dep-tree.txt | wc -l || echo "0")
    
    if [ "$WINDOWS_DUPES" -gt 3 ]; then
        print_warning "High Windows ecosystem duplication detected ($WINDOWS_DUPES instances)"
    fi
    
    if [ "$SERDE_DUPES" -gt 5 ]; then
        print_warning "High serde ecosystem duplication detected ($SERDE_DUPES instances)"
    fi
    
    print_status 0 "Dependency duplication checks"
else
    print_status 1 "Dependency duplication checks"
fi

echo ""
echo "5. Testing MSRV compliance (if Rust 1.89 is available)..."
if rustup toolchain list | grep -q "1.89.0"; then
    if rustup default 1.89.0 && cargo check --all-features; then
        print_status 0 "MSRV compliance check"
    else
        print_status 1 "MSRV compliance check"
    fi
    # Restore default toolchain
    rustup default stable
else
    print_warning "Rust 1.89.0 not available locally - skipping MSRV check"
fi

echo ""
echo "6. Testing security audit (if cargo-audit is available)..."
if command -v cargo-audit &> /dev/null; then
    if cargo audit; then
        print_status 0 "Security audit"
    else
        print_warning "Security audit found issues (review output)"
    fi
else
    print_warning "cargo-audit not available - skipping security audit"
fi

echo ""
echo "=== All dependency guardrails tests completed ==="
echo ""
echo "To run the full CI dependency management workflow:"
echo "  gh workflow run dependency-management"
echo ""
echo "To run specific jobs:"
echo "  gh workflow run dependency-management -f scan_mode=security-only"
echo "  gh workflow run dependency-management -f scan_mode=duplicates-only"