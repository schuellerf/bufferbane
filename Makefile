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

.PHONY: all build test clean install uninstall install-service uninstall-service help

# Default target
all: build

# Build the project
build:
	@echo "Building Bufferbane (release mode)..."
	cargo build $(RELEASE_FLAGS)
	@echo "Build complete: $(BUILD_DIR)/bufferbane"

# Run tests
test:
	@echo "Running tests..."
	cargo test

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Install binary and config template
install: build
	@echo "Installing Bufferbane..."
	@echo "  Installing binary to $(BINDIR)..."
	install -d $(BINDIR)
	install -m 755 $(BUILD_DIR)/bufferbane $(BINDIR)/bufferbane
	
	@echo "  Installing configuration template to $(DATADIR)..."
	install -d $(DATADIR)
	install -m 644 client.conf.template $(DATADIR)/client.conf.template
	
	@echo ""
	@echo "Bufferbane installed successfully!"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Create config: cp $(DATADIR)/client.conf.template /etc/bufferbane/client.conf"
	@echo "  2. Edit config: sudo nano /etc/bufferbane/client.conf"
	@echo "  3. Install service: sudo make install-service"
	@echo "  4. Start service: sudo systemctl start bufferbane"

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

# Show help
help:
	@echo "Bufferbane - Network Quality Monitoring"
	@echo ""
	@echo "Available targets:"
	@echo "  make build              Build the project (release mode)"
	@echo "  make test               Run tests"
	@echo "  make clean              Clean build artifacts"
	@echo "  make install            Install binary and config template"
	@echo "  make install-service    Install systemd service (requires root)"
	@echo "  make uninstall-service  Uninstall systemd service (requires root)"
	@echo "  make uninstall          Uninstall everything (requires root)"
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

