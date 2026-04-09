# ================================
# CONFIG
# ================================
APP_NAME=tasklog
ENTRY=cli.ts
BUILD_DIR=dist
BIN=$(BUILD_DIR)/$(APP_NAME)
INSTALL_DIR=$$HOME/.local/bin

BUN=bun


# ================================
# HELP
# ================================
.PHONY: help
help:
	@echo ""
	@echo "Tasklog CLI - Makefile"
	@echo ""
	@echo "Usage:"
	@echo "  make dev               Run in development mode"
	@echo "  make build             Build binary"
	@echo "  make release           Build optimized binary"
	@echo "  make install           Install globally and enable user service"
	@echo "  make uninstall         Uninstall globally and remove user service"
	@echo "  make clean             Remove build artifacts"
	@echo "  make run               Run compiled binary"
	@echo "  make doctor            Run system check"
	@echo "  make export            Export logs"
	@echo "  make service-install   Install as user service (run on boot)"
	@echo "  make service-uninstall Remove user service"
	@echo ""

# ================================
# SYSTEMD USER SERVICE
# ================================
.PHONY: service-install
service-install:
	@echo "Installing systemd user service..."
	mkdir -p $$HOME/.config/systemd/user
	cp tasklog.service $$HOME/.config/systemd/user/tasklog.service
	systemctl --user daemon-reload
	systemctl --user enable --now tasklog.service
	echo "Service installed and started."

.PHONY: service-uninstall
service-uninstall:
	@echo "Uninstalling systemd user service..."
	systemctl --user stop tasklog.service || true
	systemctl --user disable tasklog.service || true
	rm -f $$HOME/.config/systemd/user/tasklog.service
	systemctl --user daemon-reload
	echo "Service uninstalled."

# ================================
# HELP (append service usage)
# ================================
	@echo "  make install    Install globally"
	@echo "  make uninstall  Uninstall globally"
	@echo "  make clean      Remove build artifacts"
	@echo "  make run        Run compiled binary"
	@echo "  make doctor     Run system check"
	@echo ""

# ================================
# DEVELOPMENT
# ================================
.PHONY: dev
dev:
	$(BUN) run $(ENTRY) run

# ================================
# BUILD
# ================================
.PHONY: build
build:
	@echo "Building..."
	mkdir -p $(BUILD_DIR)
	$(BUN) build $(ENTRY) --compile --outfile $(BIN)

# optimized production build
.PHONY: release
release:
	@echo "Building (release mode)..."
	mkdir -p $(BUILD_DIR)
	$(BUN) build $(ENTRY) \
		--compile \
		--minify \
		--sourcemap=none \
		--outfile $(BIN)

# ================================
# RUN
# ================================
.PHONY: run
run: build
	./$(BIN) run

# ================================
# INSTALL
# ================================
.PHONY: install
install: release service-install
	@echo "Installing to user directory..."
	mkdir -p $(INSTALL_DIR)
	cp $(BIN) $(INSTALL_DIR)/$(APP_NAME)
	chmod +x $(INSTALL_DIR)/$(APP_NAME)

	@echo ""
	@echo "Installed to $(INSTALL_DIR)/$(APP_NAME)"

	@if ! echo $$PATH | grep -q "$(INSTALL_DIR)"; then \
		echo ""; \
		echo "WARNING: $(INSTALL_DIR) is not in your PATH"; \
		echo "Add this to your shell config (~/.bashrc, ~/.zshrc):"; \
		echo "  export PATH=\$$HOME/.local/bin:\$$PATH"; \
	fi

# ================================
# UNINSTALL
# ================================
.PHONY: uninstall
uninstall: service-uninstall
	@echo "Uninstalling $(APP_NAME)..."
	@if [ -f $(INSTALL_DIR)/$(APP_NAME) ]; then \
		rm $(INSTALL_DIR)/$(APP_NAME); \
		echo "Removed from $(INSTALL_DIR)"; \
	else \
		echo "Not installed"; \
	fi

# ================================
# CLEAN
# ================================
.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)

# ================================
# UTILITIES
# ================================
.PHONY: doctor
doctor:
	$(BUN) run $(ENTRY) doctor

.PHONY: export
export:
	$(BUN) run $(ENTRY) export
