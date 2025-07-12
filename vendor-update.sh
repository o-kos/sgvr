#!/bin/bash

# Script for automatically updating dependencies in a "vendored" Rust project.
#
# Usage:
#   ./update_vendor.sh add <crate-name> [other cargo add flags]
#   ./update_vendor.sh update [other cargo update flags]
#
# Examples:
#   ./update_vendor.sh add serde --features derive
#   ./update_vendor.sh update -p tokio
#   ./update_vendor.sh update

# 1. Stop execution on any error
set -e

# 2. Check that the script is run from project root and arguments are provided
if [ ! -f "Cargo.toml" ]; then
  echo "Error: Cargo.toml not found. Please run the script from your project's root directory."
  exit 1
fi

if [ "$#" -eq 0 ]; then
  echo "Error: no Cargo command specified."
  echo "Usage: $0 <add|update> [arguments...]"
  echo "Example: $0 add serde --features derive"
  exit 1
fi

CONFIG_FILE=".cargo/config.toml"

# --- STEP 1: Temporarily disable offline mode ---
echo "--> Step 1: Temporarily disabling offline mode..."
if [ ! -f "$CONFIG_FILE" ]; then
  echo "Warning: $CONFIG_FILE not found. Skipping this step."
  echo "The project may not have been vendored yet."
else
  mv "$CONFIG_FILE" "$CONFIG_FILE.bak"
  echo "File $CONFIG_FILE renamed to $CONFIG_FILE.bak"
fi
echo ""

# --- STEP 2: Add or update dependencies ---
CARGO_COMMAND=$1
shift # Shift arguments so $@ contains only cargo parameters
echo "--> Step 2: Executing 'cargo $CARGO_COMMAND $@'..."
cargo "$CARGO_COMMAND" "$@"
echo ""

# --- STEP 3: Update vendor directory contents ---
echo "--> Step 3: Updating local sources (vendoring)..."
# Remove old directory if it exists to avoid garbage
if [ -d "vendor" ]; then
  echo "Removing old vendor directory..."
  rm -rf vendor
fi
cargo vendor
echo "Vendor directory successfully created/updated."
echo ""

# --- STEP 4: Restore offline mode ---
echo "--> Step 4: Restoring offline mode..."
if [ ! -f "$CONFIG_FILE.bak" ]; then
  echo "Warning: $CONFIG_FILE.bak not found. Skipping."
else
  mv "$CONFIG_FILE.bak" "$CONFIG_FILE"
  echo "Offline mode configuration restored."
fi
echo ""

# --- STEP 5: Verification ---
echo "--> Step 5: Testing offline build..."
cargo build --offline
echo "Offline build test passed successfully!"
echo ""

# --- STEP 6: Commit changes to Git ---
echo "--> Step 6: Preparing for Git commit."
echo "Changes ready for commit:"
git status -s # Show brief status
echo ""

read -p "Do you want to commit these changes now? (y/N) " -n 1 -r
echo # Move to new line after input

if [[ $REPLY =~ ^[Yy]$ ]]; then
  read -p "Enter commit message (or press Enter for default): " COMMIT_MESSAGE
  if [ -z "$COMMIT_MESSAGE" ]; then
    COMMIT_MESSAGE="build: Update vendored dependencies"
  fi

  echo "Adding files to index: Cargo.toml, Cargo.lock, vendor/, .cargo/config.toml"
  # Add all necessary files, even if they haven't changed (git will figure it out)
  git add Cargo.toml Cargo.lock vendor/ .cargo/config.toml

  echo "Committing with message: '$COMMIT_MESSAGE'"
  git commit -m "$COMMIT_MESSAGE"
  echo "Changes successfully committed."
else
  echo "Commit skipped. Please review and commit changes manually."
fi

echo ""
echo "✅ All steps completed."