.PHONY: all build install


build:
	cargo build --release

install: build
	cargo install
	systemctl --user stop mcenroe.service
	systemctl --user disable mcenroe.service
	cp mcenroe.service ~/.config/systemd/user/.
	systemctl --user start mcenroe.service
	systemctl --user enable mcenroe.service
  

all: build install
