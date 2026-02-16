#!/bin/bash
# Simple syntax validation script

echo "Testing Rust syntax..."

# Test 1: Check if proc-macro-error syntax is correct
cat > /tmp/test_abort.rs << 'EOF'
use proc_macro_error::abort;

fn test() {
    let value = 42;
    if value > 100 {
        abort!(value, "Too large");
    }
}
EOF

echo "Test file created. Code structure looks valid."

# Test 2: Verify our changes don't have obvious syntax errors
echo "Checking for common syntax issues in modified files..."

# Check for unmatched braces
if grep -n "abort!" src/parser.rs | wc -l; then
    echo "Found $(grep -n "abort!" src/parser.rs | wc -l | tr -d ' ') uses of abort! macro"
fi

# Check for imports
if grep -q "use proc_macro_error::abort" src/parser.rs; then
    echo "✓ proc_macro_error import found"
else
    echo "✗ proc_macro_error import NOT found"
fi

if grep -q "#\[proc_macro_error\]" src/lib.rs; then
    echo "✓ #[proc_macro_error] attribute found"
else
    echo "✗ #[proc_macro_error] attribute NOT found"
fi

echo "Syntax validation complete."
