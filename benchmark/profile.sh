#!/usr/bin/env bash
set -euo pipefail

export RB_SYS_CARGO_PROFILE=profiling

echo "📦 Installing Ruby dependencies..."
bundle install

echo "🔨 Compiling Rust extension..."
bundle exec rake compile

# cargo install flamegraph
sudo flamegraph -o flamegraph.svg -- bundle exec benchmark/ruby_profiling_script.rb
