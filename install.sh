#!/bin/bash

# Function to check Rust version
check_rust_version() {
    local version=$(rustc --version | cut -d ' ' -f2)
    local required_version="1.74.0"

    if [[ "$(printf '%s\n' "$required_version" "$version" | sort -V | head -n1)" != "$required_version" ]]; then
        echo "Error: Rust version $required_version or newer is required. Please update Rust and try again."
        exit 1
    fi
}

# Function to download and install Bott
install_bott() {
    local temp_dir=$(mktemp -d)
    local bott_dir="$HOME/.bott"
    local bott_repo="https://github.com/subbu963/bott"

    # Check Rust version
    check_rust_version

    # Clone Bott repository
    git clone "$bott_repo" "$temp_dir" || {
        echo "Error: Failed to clone Bott repository. Please check your internet connection and try again."
        exit 1
    }

    # Compile the code
    cd "$temp_dir" || exit 1
    cargo build --release || {
        echo "Error: Failed to compile Bott. Please check for any compilation errors and try again."
        exit 1
    }

    # Create Bott directory
    mkdir -p "$bott_dir"

    # Copy binaries to Bott directory
    cp "$temp_dir/target/release/bott" "$bott_dir/"
    cp "$temp_dir/bott.sh" "$bott_dir/"

    # Prompt user to update shell configuration
    echo "Bott installed successfully!"
    echo "Please append the following lines to your shell configuration file (.bashrc, .zshrc, etc.):"
    echo ''
    echo "export BOTT_DIR=\"$bott_dir\""
    echo '[ -s "$BOTT_DIR/bott.sh" ] && \. "$BOTT_DIR/bott.sh"  # This loads bott'
    echo ''
}

# Main execution
install_bott
