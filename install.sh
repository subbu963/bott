#!/bin/bash

# Check if Rust is installed and the version is greater than or equal to 1.74.0
if command -v rustc >/dev/null 2>&1 && [ "$(rustc --version | awk '{print $2}')" \> "1.74.0" ]; then
  echo "Rust is installed, and the version is greater than or equal to 1.74.0."
else
  echo "Installing Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source $HOME/.cargo/env
fi

# Create a temporary directory
TEMP=$(mktemp -d)

# Clone the repository
git clone https://github.com/subbu963/bott.git "$TEMP"

# Change to the project directory
cd "$TEMP"

# Compile the code using cargo
cargo build --release

# Create the .bott directory in the home directory
mkdir -p "$HOME/.bott"

# Copy the compiled binary to .bott directory
cp "$TEMP/target/release/bott" "$HOME/.bott/"

# Copy the bott.sh script to .bott directory
cp "$TEMP/bott.sh" "$HOME/.bott/"

# Clean up temporary directory
rm -rf "$TEMP"

# Prompt user to append configuration snippet to their shell configuration file
echo "Please append the following snippet to your shell configuration file (.bashrc, .zshrc, etc.):"
echo ""
echo 'export BOTT_DIR="$HOME/.bott"'
echo '[ -s "$BOTT_DIR/bott.sh" ] && \. "$BOTT_DIR/bott.sh"  # This loads bott'
echo ""
echo "Installation completed successfully."
