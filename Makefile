.PHONY: all install

install:
	cargo install --path .
	systemctl --user stop mcenroe.service || echo
	systemctl --user disable mcenroe.service || echo
	cp mcenroe.service ~/.config/systemd/user/.
	systemctl --user start mcenroe.service
	systemctl --user enable mcenroe.service
  

all: install
