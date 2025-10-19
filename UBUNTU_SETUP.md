# Ubuntu Setup Guide - ICMP Without Sudo

## Problem

```
Error: Failed to create ICMP client (CAP_NET_RAW required)
Caused by: Operation not permitted (os error 1)
```

This occurs because ICMP (ping) requires raw socket access, which needs elevated privileges.

## Why Does This Work Out of the Box on Fedora?

**TL;DR:** Fedora enables unprivileged ICMP sockets by default, Ubuntu doesn't.

Your Fedora system has:
```bash
$ cat /proc/sys/net/ipv4/ping_group_range
0	2147483647
```

This means **all users (GID 0 to max)** can create ICMP sockets without special privileges.

Ubuntu's default is:
```bash
$ cat /proc/sys/net/ipv4/ping_group_range
1	0
```

This effectively **disables** unprivileged ICMP sockets (1 > 0 = no valid range).

### Why the difference?

- **Fedora**: Security through modern kernel features, more permissive defaults for developer tools
- **Ubuntu**: More conservative defaults, requires explicit permission grants via `setcap`

Both approaches are valid security models. Fedora assumes the kernel's unprivileged ICMP implementation is secure enough. Ubuntu prefers explicit capability grants.

## Solutions for Ubuntu

### Solution A: Enable System-Wide Like Fedora (Easiest)

Make Ubuntu behave like Fedora by enabling unprivileged ICMP:

```bash
# Enable for all users (temporary, lost on reboot)
sudo sysctl -w net.ipv4.ping_group_range="0 2147483647"

# Make permanent (survives reboot)
echo "net.ipv4.ping_group_range = 0 2147483647" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

**Advantages:**
- ✅ Works for all users and all applications
- ✅ No need to re-apply after rebuilding
- ✅ Same behavior as Fedora
- ✅ Uses modern kernel ICMP socket support

**One-time setup**, then bufferbane works without any special permissions.

### Solution B: Grant CAP_NET_RAW to Bufferbane Binary

Alternatively, grant permission only to the bufferbane binary:

#### For Installed Binary

If you've installed bufferbane system-wide:

```bash
# Grant CAP_NET_RAW capability to the installed binary
sudo setcap cap_net_raw+ep /usr/local/bin/bufferbane

# Verify it was set correctly
getcap /usr/local/bin/bufferbane
# Should output: /usr/local/bin/bufferbane = cap_net_raw+ep
```

Now you can run bufferbane as a regular user:
```bash
bufferbane monitor
```

#### For Development Binary

If you're testing the binary in your build directory:

```bash
# Grant capability to the release binary
sudo setcap cap_net_raw+ep ./target/release/bufferbane

# Verify
getcap ./target/release/bufferbane

# Run without sudo
./target/release/bufferbane monitor
```

#### For Static Build

If you're using the static musl build:

```bash
# Grant capability to static binary
sudo setcap cap_net_raw+ep ./target/x86_64-unknown-linux-musl/release/bufferbane

# Verify
getcap ./target/x86_64-unknown-linux-musl/release/bufferbane

# Run without sudo
./target/x86_64-unknown-linux-musl/release/bufferbane monitor
```

**Drawbacks of Solution B:**
- ❌ Must re-apply after each rebuild
- ❌ Only works for the specific binary (not other tools)
- ❌ Must be set for each copy of the binary

## Important Notes

### Capabilities Persist

The capability setting:
- ✅ **Persists** across reboots
- ✅ **Survives** file moves (on same filesystem)
- ❌ **Lost** when you rebuild the binary
- ❌ **Lost** when you copy the file to another filesystem

### After Rebuilding

Every time you rebuild, you need to reapply:
```bash
cargo build --release
sudo setcap cap_net_raw+ep ./target/release/bufferbane
```

### Automatic Setup with Make

Use the Makefile target that handles this automatically:

```bash
# Build and install with capability
sudo make install
sudo make install-service

# The install-service target automatically sets CAP_NET_RAW
```

From the Makefile:
```makefile
install-service:
    # ... install service file ...
    @echo "  Setting CAP_NET_RAW capability on binary..."
    setcap cap_net_raw+ep $(BINDIR)/bufferbane
```

## Verification

### Check if capability is set:
```bash
getcap /usr/local/bin/bufferbane
# Expected output: /usr/local/bin/bufferbane = cap_net_raw+ep
```

### Test ICMP functionality:
```bash
bufferbane monitor
# Should start monitoring without errors
```

### Check what capabilities the process has:
```bash
# While bufferbane is running:
ps aux | grep bufferbane
# Get the PID, then:
grep Cap /proc/<PID>/status
```

## Which Solution Should You Use?

### For Personal/Development Machines: Use Solution A

**Recommended:** Enable system-wide unprivileged ICMP (Solution A) because:
- ✅ One-time setup
- ✅ Works for all tools (ping, mtr, bufferbane, etc.)
- ✅ No need to remember to re-apply after rebuilds
- ✅ Same as Fedora's default behavior

### For Production/Shared Systems: Use Solution B

Use `setcap` (Solution B) if:
- You want to limit ICMP to specific binaries only
- You're following a security policy that requires explicit grants
- You don't have permission to change kernel parameters

## Security Considerations

### Is Solution A (unprivileged ICMP) safe?

**Yes.** This is Fedora's default and has been available since Linux kernel 3.0 (2011).

**How it works:**
- Uses kernel's built-in ICMP socket type (`SOCK_DGRAM` with `IPPROTO_ICMP`)
- Does NOT grant raw socket access
- Limited to ICMP Echo Request/Reply only
- Cannot craft arbitrary network packets
- No special privileges required

**Why it's secure:**
- ✅ Sandboxed by kernel - users can only send/receive ICMP echo
- ✅ Cannot inspect other users' traffic
- ✅ Cannot spoof source addresses
- ✅ Used by Red Hat/Fedora for 10+ years without issues

### What does CAP_NET_RAW allow? (Solution B)

The `CAP_NET_RAW` capability allows the binary to:
- ✅ Send ICMP packets (ping)
- ✅ Create raw sockets
- ✅ Bind to any local address
- ✅ Craft custom network packets
- ❌ **Does NOT** grant full root access
- ❌ **Does NOT** allow file system access beyond user permissions

### Is Solution B safe?

Yes, granting `CAP_NET_RAW` to bufferbane is safe because:
1. It only allows network packet operations
2. It doesn't grant write access to system files
3. It's limited to what the binary needs for monitoring
4. It's the standard approach for network monitoring tools

**Note:** `CAP_NET_RAW` is more powerful than unprivileged ICMP (Solution A), but bufferbane only uses it for ICMP echo requests.

### Who should use this?

This is appropriate for:
- ✅ Personal workstations
- ✅ Development machines
- ✅ Monitoring servers
- ✅ Systems where you trust the binary

## Troubleshooting

### "Operation not permitted" when setting capability

You need sudo to set capabilities:
```bash
# Wrong:
setcap cap_net_raw+ep ./bufferbane

# Correct:
sudo setcap cap_net_raw+ep ./bufferbane
```

### Capability not persisting

Capabilities are stored in extended file attributes. Ensure:
```bash
# Check if filesystem supports extended attributes
mount | grep /home
# Should show 'user_xattr' or be ext4/xfs/btrfs

# If on a filesystem that doesn't support xattr, you'll need to:
# 1. Run as root, or
# 2. Move the binary to a supported filesystem
```

### Capability lost after rebuild

This is expected. After `cargo build`, re-apply:
```bash
sudo setcap cap_net_raw+ep ./target/release/bufferbane
```

Or use the install target:
```bash
sudo make install
```

### Still getting permission denied

Check that your user can execute the binary:
```bash
ls -l /usr/local/bin/bufferbane
# Should be: -rwxr-xr-x

# If not executable:
sudo chmod +x /usr/local/bin/bufferbane
sudo setcap cap_net_raw+ep /usr/local/bin/bufferbane
```

## Quick Reference

### Solution A: Enable System-Wide (Recommended for Development)

```bash
# One-time setup (make permanent):
echo "net.ipv4.ping_group_range = 0 2147483647" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p

# Now bufferbane works without any special setup:
cargo build --release
./target/release/bufferbane monitor
```

### Solution B: Per-Binary Capability

```bash
# Development workflow:
cargo build --release
sudo setcap cap_net_raw+ep ./target/release/bufferbane
./target/release/bufferbane monitor

# Production installation:
sudo make install
sudo make install-service
systemctl start bufferbane

# Check capability:
getcap /usr/local/bin/bufferbane

# Remove capability (if needed):
sudo setcap -r /usr/local/bin/bufferbane
```

## See Also

- `man capabilities` - Full documentation on Linux capabilities
- `man setcap` - Setting file capabilities
- `man getcap` - Querying file capabilities
- [Ubuntu Manpage: capabilities](http://manpages.ubuntu.com/manpages/focal/man7/capabilities.7.html)

