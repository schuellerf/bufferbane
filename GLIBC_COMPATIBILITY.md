# GLIBC Compatibility Issue - Solution Guide

## Problem

When deploying `bufferbane-server` to older Linux distributions, you may encounter:

```
./bufferbane-server: /lib/x86_64-linux-gnu/libm.so.6: version `GLIBC_2.29' not found
```

**Root cause**: The binary was compiled on a system with newer GLIBC (e.g., Fedora 42 with GLIBC 2.39) but your target server has an older GLIBC version.

---

## Solution 1: Use setup-server.sh (Recommended)

The setup script **automatically** builds with musl for maximum compatibility:

```bash
./setup-server.sh
```

This will:
1. Detect if musl target is available
2. Install it if needed (via rustup)
3. Build a fully static binary that works on **any** Linux

---

## Solution 2: Manual Build with Musl

### On Fedora (your build machine):

```bash
# Option A: Install rustup for musl support
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup target add x86_64-unknown-linux-musl

# Option B: Use musl-gcc (if system Rust)
sudo dnf install musl-gcc musl-devel

# Build static binary
make build-server-static

# Or manually:
cargo build --release --target x86_64-unknown-linux-musl -p bufferbane-server

# Result: target/x86_64-unknown-linux-musl/release/bufferbane-server
```

### Deploy the static binary:

```bash
scp target/x86_64-unknown-linux-musl/release/bufferbane-server user@server:/opt/bufferbane/
```

---

## Solution 3: Compile on Target Machine

If musl doesn't work, compile directly on the Debian server:

```bash
# On your Debian server
sudo apt update
sudo apt install -y build-essential rust-all

# Transfer source code
scp -r /path/to/bufferbane user@server:/tmp/

# On server: build
cd /tmp/bufferbane
cargo build --release -p bufferbane-server

# Install
sudo cp target/release/bufferbane-server /opt/bufferbane/
```

---

## Solution 4: Use Docker Multi-stage Build

Build in a container with old GLIBC:

```dockerfile
# Dockerfile.server
FROM debian:11 as builder
RUN apt update && apt install -y build-essential rust-all
WORKDIR /build
COPY . .
RUN cargo build --release -p bufferbane-server

FROM debian:11
COPY --from=builder /build/target/release/bufferbane-server /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/bufferbane-server"]
```

```bash
docker build -f Dockerfile.server -t bufferbane-server .
docker run -p 9876:9876/udp bufferbane-server --config server.conf
```

---

## Verification

After deploying, verify it works:

```bash
# On Debian server
./bufferbane-server --version

# Should output:
# bufferbane-server 0.1.0

# Test with config
./bufferbane-server --config server.conf

# Should output:
# INFO Starting Bufferbane server v0.1.0
# INFO Server listening on 0.0.0.0:9876
```

---

## Why Musl is Recommended

**Advantages**:
- ✅ **Fully static**: No GLIBC dependency
- ✅ **Universal**: Works on any Linux (even Alpine)
- ✅ **Small**: ~3-4MB binary
- ✅ **Portable**: Copy anywhere and run

**Disadvantages**:
- Slightly larger binary (~10% bigger)
- Slightly slower at runtime (~5% slower, negligible for network I/O)

**For a network monitoring server, musl is the best choice!**

---

## Quick Reference

### Check GLIBC version on target:

```bash
ldd --version
# Or:
/lib/x86_64-linux-gnu/libc.so.6
```

### Build commands comparison:

```bash
# Standard build (GLIBC-dependent)
cargo build --release -p bufferbane-server
# Binary: target/release/bufferbane-server
# Size: ~3.2MB
# Works on: Same/newer GLIBC only

# Static build (musl, portable)
cargo build --release --target x86_64-unknown-linux-musl -p bufferbane-server
# Binary: target/x86_64-unknown-linux-musl/release/bufferbane-server  
# Size: ~3.5MB
# Works on: Any Linux
```

---

## Troubleshooting

### "musl-gcc: command not found"

```bash
# Fedora
sudo dnf install musl-gcc musl-devel

# Debian/Ubuntu
sudo apt install musl-tools musl-dev
```

### "target 'x86_64-unknown-linux-musl' not found"

```bash
# Install via rustup
rustup target add x86_64-unknown-linux-musl

# Or install system Rust with musl support
# Fedora: (need rustup)
# Debian/Ubuntu:
sudo apt install rustc cargo
cargo build --target x86_64-unknown-linux-musl -p bufferbane-server
```

### "cannot compile proc-macro with musl"

This is a known issue with some dependencies. Solution:

1. Use setup-server.sh (handles this automatically)
2. Or build with `--target-dir` workaround:
   ```bash
   cargo build --release --target x86_64-unknown-linux-musl \
     --target-dir target-musl -p bufferbane-server
   ```

---

## Automated Solution (Recommended)

**Just use the setup script - it handles everything!**

```bash
./setup-server.sh
# Follow prompts
# Binary automatically built with musl for maximum compatibility
# Deployed to your server
# Ready to use!
```

---

## For Your Immediate Issue

Since you already have a GLIBC error on Debian:

**Quick fix**:

```bash
# 1. On your build machine (Fedora), rebuild with musl:
./setup-server.sh
# Enter your server details when prompted
# Script will build and deploy static binary

# 2. Or manually:
cargo build --release --target x86_64-unknown-linux-musl -p bufferbane-server
scp target/x86_64-unknown-linux-musl/release/bufferbane-server user@debian:/opt/bufferbane/

# 3. On Debian server:
cd /opt/bufferbane
./bufferbane-server --config server.conf
# Should work now!
```

---

**Next time**: Always use `./setup-server.sh` for deployment - it builds with musl automatically! 🎯

