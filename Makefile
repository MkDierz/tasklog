APP_NAME=tasklog
CARGO=cargo
INSTALL_DIR=$$HOME/.local/bin

.PHONY: help
help:
	@echo ""
	@echo "Tasklog CLI - Rust build"
	@echo ""
	@echo "Usage:"
	@echo "  make build             Build the binary"
	@echo "  make run               Run the binary"
	@echo "  make release           Build optimized release binary"
	@echo "  make install           Install to ~/.local/bin"
	@echo "  make clean             Remove build artifacts"
	@echo "  make doctor            Run system check"
	@echo "  make export            Export logs"
	@echo "  make service-install   Install the user service"
	@echo "  make service-uninstall Remove the user service"
	@echo ""

.PHONY: build
build:
	$(CARGO) build

.PHONY: release
release:
	$(CARGO) build --release

.PHONY: run
run:
	$(CARGO) run -- run

.PHONY: install
install: release service-install
	mkdir -p $(INSTALL_DIR)
	cp target/release/$(APP_NAME) $(INSTALL_DIR)/$(APP_NAME)
	chmod +x $(INSTALL_DIR)/$(APP_NAME)

.PHONY: uninstall
uninstall: service-uninstall
	@if [ -f $(INSTALL_DIR)/$(APP_NAME) ]; then rm $(INSTALL_DIR)/$(APP_NAME); fi

.PHONY: clean
clean:
	$(CARGO) clean

.PHONY: doctor
doctor:
	$(CARGO) run -- doctor

.PHONY: export
export:
	$(CARGO) run -- export

.PHONY: service-install
service-install:
	@echo "Installing systemd user service..."
	mkdir -p $$HOME/.config/systemd/user
	cp tasklog.service $$HOME/.config/systemd/user/tasklog.service
	systemctl --user daemon-reload
	systemctl --user enable --now tasklog.service

.PHONY: service-uninstall
service-uninstall:
	@echo "Uninstalling systemd user service..."
	systemctl --user stop tasklog.service || true
	systemctl --user disable tasklog.service || true
	rm -f $$HOME/.config/systemd/user/tasklog.service
	systemctl --user daemon-reload
