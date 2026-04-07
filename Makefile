# ================================
# CONFIG
# ================================
APP_NAME=tasklog
ENTRY=cli.ts
BUILD_DIR=dist
BIN=$(BUILD_DIR)/$(APP_NAME)

BUN=bun

# ================================
# DEFAULT
# ================================
.PHONY: help
help:
	@echo ""
	@echo "Tasklog CLI - Makefile"
	@echo ""
	@echo "Usage:"
	@echo "  make dev        Run in development mode"
	@echo "  make build      Build binary"
	@echo "  make release    Build optimized binary"
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
install: release
	@echo "Installing globally..."
	cp $(BIN) /usr/local/bin/$(APP_NAME)
	chmod +x /usr/local/bin/$(APP_NAME)

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


# ================================
# UNINSTALL
# ================================
.PHONY: uninstall
uninstall:
	@echo "Uninstalling $(APP_NAME)..."
	@if [ -f /usr/local/bin/$(APP_NAME) ]; then \
		rm /usr/local/bin/$(APP_NAME); \
		echo "Removed"; \
	else \
		echo "Not installed"; \
	fi