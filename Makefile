# Bufferbane - Network Quality Monitoring
# Makefile for building and installing

# Configuration
PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
SYSCONFDIR = /etc
SYSTEMD_UNIT_DIR = /etc/systemd/system
DATADIR = $(PREFIX)/share/bufferbane

# Build type
RELEASE_FLAGS = --release
BUILD_DIR = target/release

# Installation user/group (default: current user, or override with sudo)
INSTALL_USER ?= $(USER)
INSTALL_GROUP ?= $(shell id -gn $(INSTALL_USER))

# Detect home directory for user
USER_HOME = $(shell getent passwd $(INSTALL_USER) | cut -d: -f6)

.PHONY: all build build-client build-server test clean clean-data install uninstall install-service uninstall-service windows windows-setup help

# Default target
all: build

# Build the project
build:
	@echo "Building Bufferbane (release mode)..."
	cargo build $(RELEASE_FLAGS)
	@echo "Build complete:"
	@echo "  Client: $(BUILD_DIR)/bufferbane"
	@echo "  Server: $(BUILD_DIR)/bufferbane-server"

# Build only client
build-client:
	@echo "Building Bufferbane client (release mode)..."
	cargo build $(RELEASE_FLAGS) -p bufferbane
	@echo "Build complete: $(BUILD_DIR)/bufferbane"

# Build only server
build-server:
	@echo "Building Bufferbane server (release mode)..."
	cargo build $(RELEASE_FLAGS) -p bufferbane-server
	@echo "Build complete: $(BUILD_DIR)/bufferbane-server"

# Run tests
test:
	@echo "Running tests..."
	cargo test

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Clean generated data (database, charts, exports, logs)
clean-data:
	@echo "Cleaning generated data files..."
	@echo "  Removing database files..."
	@rm -f *.db *.db-shm *.db-wal bufferbane.db* 2>/dev/null || true
	@echo "  Removing charts..."
	@rm -f *.png *.html 2>/dev/null || true
	@echo "  Removing exports..."
	@rm -f *.csv *.json stats.json 2>/dev/null || true
	@rm -rf exports/ 2>/dev/null || true
	@echo "  Removing log files..."
	@rm -f *.log 2>/dev/null || true
	@echo "  Removing temporary files..."
	@rm -f *.tmp *.temp 2>/dev/null || true
	@echo ""
	@echo "✓ Data cleaned. Source code, templates, and config preserved."
	@echo ""
	@echo "Note: This only cleans the current directory."
	@echo "User data locations are not affected:"
	@echo "  - ~/.local/share/bufferbane/"
	@echo "  - /etc/bufferbane/"

# Install binary and config template
install: build
	@echo "Installing Bufferbane..."
	@echo "  Installing binaries to $(BINDIR)..."
	install -d $(BINDIR)
	install -m 755 $(BUILD_DIR)/bufferbane $(BINDIR)/bufferbane
	install -m 755 $(BUILD_DIR)/bufferbane-server $(BINDIR)/bufferbane-server
	
	@echo "  Installing configuration templates to $(DATADIR)..."
	install -d $(DATADIR)
	install -m 644 client.conf.template $(DATADIR)/client.conf.template
	install -m 644 server.conf.template $(DATADIR)/server.conf.template
	
	@echo ""
	@echo "Bufferbane installed successfully!"
	@echo ""
	@echo "Client setup:"
	@echo "  1. Create config: cp $(DATADIR)/client.conf.template /etc/bufferbane/client.conf"
	@echo "  2. Edit config: sudo nano /etc/bufferbane/client.conf"
	@echo "  3. Install service: sudo make install-service"
	@echo "  4. Start service: sudo systemctl start bufferbane"
	@echo ""
	@echo "Server setup (Phase 2 - optional):"
	@echo "  1. Create config: cp $(DATADIR)/server.conf.template /etc/bufferbane/server.conf"
	@echo "  2. Edit config: sudo nano /etc/bufferbane/server.conf"
	@echo "  3. Run server: bufferbane-server --config /etc/bufferbane/server.conf"

# Install systemd service
install-service:
	@if [ ! -f /etc/bufferbane/client.conf ]; then \
		echo "Error: /etc/bufferbane/client.conf not found!"; \
		echo "Please create it first:"; \
		echo "  sudo mkdir -p /etc/bufferbane"; \
		echo "  sudo cp $(DATADIR)/client.conf.template /etc/bufferbane/client.conf"; \
		echo "  sudo nano /etc/bufferbane/client.conf"; \
		exit 1; \
	fi
	
	@echo "Installing systemd service..."
	@echo "  Creating service file from template..."
	@sed -e "s|@BINDIR@|$(BINDIR)|g" \
	     -e "s|@INSTALL_USER@|$(INSTALL_USER)|g" \
	     -e "s|@USER_HOME@|$(USER_HOME)|g" \
	     bufferbane.service.in > /tmp/bufferbane.service
	
	@echo "  Installing service to $(SYSTEMD_UNIT_DIR)..."
	install -m 644 /tmp/bufferbane.service $(SYSTEMD_UNIT_DIR)/bufferbane.service
	rm /tmp/bufferbane.service
	
	@echo "  Setting CAP_NET_RAW capability on binary..."
	setcap cap_net_raw+ep $(BINDIR)/bufferbane || echo "Warning: setcap failed. Service may need to run as root."
	
	@echo "  Reloading systemd..."
	systemctl daemon-reload
	
	@echo ""
	@echo "Systemd service installed successfully!"
	@echo ""
	@echo "Next steps:"
	@echo "  Enable service: sudo systemctl enable bufferbane"
	@echo "  Start service:  sudo systemctl start bufferbane"
	@echo "  Check status:   sudo systemctl status bufferbane"
	@echo "  View logs:      sudo journalctl -u bufferbane -f"

# Uninstall systemd service
uninstall-service:
	@echo "Uninstalling systemd service..."
	@if systemctl is-active --quiet bufferbane; then \
		echo "  Stopping service..."; \
		systemctl stop bufferbane; \
	fi
	@if systemctl is-enabled --quiet bufferbane 2>/dev/null; then \
		echo "  Disabling service..."; \
		systemctl disable bufferbane; \
	fi
	@if [ -f $(SYSTEMD_UNIT_DIR)/bufferbane.service ]; then \
		echo "  Removing service file..."; \
		rm -f $(SYSTEMD_UNIT_DIR)/bufferbane.service; \
	fi
	@echo "  Reloading systemd..."
	systemctl daemon-reload
	@echo "Service uninstalled."

# Uninstall everything
uninstall: uninstall-service
	@echo "Uninstalling Bufferbane..."
	@if [ -f $(BINDIR)/bufferbane ]; then \
		echo "  Removing binary..."; \
		rm -f $(BINDIR)/bufferbane; \
	fi
	@if [ -d $(DATADIR) ]; then \
		echo "  Removing data directory..."; \
		rm -rf $(DATADIR); \
	fi
	@echo "Bufferbane uninstalled."
	@echo ""
	@echo "Note: Configuration in /etc/bufferbane/ was not removed."
	@echo "To remove it: sudo rm -rf /etc/bufferbane/"

# Cross-compile for Windows (requires mingw-w64)
windows-setup:
	@echo "Installing Windows cross-compilation prerequisites..."
	@echo ""
	@if command -v rustup >/dev/null 2>&1; then \
		echo "  [rustup detected] Adding Rust target for Windows..."; \
		rustup target add x86_64-pc-windows-gnu; \
	elif [ -f /etc/fedora-release ] || [ -f /etc/redhat-release ]; then \
		echo "  [Fedora/RHEL detected] Installing Rust Windows target via dnf..."; \
		echo "  Run: sudo dnf install rust-std-static-x86_64-pc-windows-gnu"; \
	elif [ -f /etc/debian_version ]; then \
		echo "  [Debian/Ubuntu detected] Installing Rust Windows target..."; \
		echo "  Run: sudo apt install rustc-targets-x86-64-pc-windows-gnu"; \
	else \
		echo "  Please install Rust Windows target manually for your system"; \
	fi
	@echo ""
	@echo "You also need mingw-w64 cross-compiler. Install with:"
	@echo "  Fedora: sudo dnf install mingw64-gcc mingw64-winpthreads-static"
	@echo "  Ubuntu: sudo apt install mingw-w64"
	@echo "  Arch:   sudo pacman -S mingw-w64-gcc"

windows:
	@echo "Cross-compiling Bufferbane for Windows..."
	@echo "  Target: x86_64-pc-windows-gnu"
	@echo ""
	cargo build --release --target x86_64-pc-windows-gnu
	@echo ""
	@echo "Windows build complete!"
	@echo "Binary: target/x86_64-pc-windows-gnu/release/bufferbane.exe"
	@echo ""
	@echo "Transfer to Windows and run (requires Administrator for ICMP):"
	@echo "  bufferbane.exe --config client.conf"
	@echo "  bufferbane.exe --chart --last 24h --output latency.png"
	@echo ""
	@echo "Note: Windows builds have limited functionality:"
	@echo "  - ICMP requires Administrator privileges"
	@echo "  - No systemd service support"
	@echo "  - Chart export and database features work normally"

# Show help
help:
	@echo "Bufferbane - Network Quality Monitoring"
	@echo ""
	@echo "Available targets:"
	@echo "  make build              Build client and server (release mode)"
	@echo "  make build-client       Build only the client"
	@echo "  make build-server       Build only the server (Phase 2)"
	@echo "  make test               Run tests"
	@echo "  make clean              Clean build artifacts"
	@echo "  make clean-data         Clean generated data (db, charts, exports, logs)"
	@echo "  make install            Install binaries and config templates"
	@echo "  make install-service    Install systemd service (requires root)"
	@echo "  make uninstall-service  Uninstall systemd service (requires root)"
	@echo "  make uninstall          Uninstall everything (requires root)"
	@echo "  make windows-setup      Install Windows cross-compilation tools"
	@echo "  make windows            Cross-compile for Windows (x64)"
	@echo "  make help               Show this help message"
	@echo ""
	@echo "Quick installation:"
	@echo "  sudo make install"
	@echo "  sudo mkdir -p /etc/bufferbane"
	@echo "  sudo cp $(DATADIR)/client.conf.template /etc/bufferbane/client.conf"
	@echo "  sudo nano /etc/bufferbane/client.conf"
	@echo "  sudo make install-service"
	@echo "  sudo systemctl enable --now bufferbane"
	@echo ""
	@echo "Configuration:"
	@echo "  PREFIX=/usr/local       Installation prefix (default: /usr/local)"
	@echo "  INSTALL_USER=username   User to run service as (default: current user)"

