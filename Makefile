ARTIFACTS := artifacts

COUNTRY_MMDB := $(ARTIFACTS)/Country.mmdb
GEOSITE_DAT  := $(ARTIFACTS)/geosite.dat
GEOIP_DAT    := $(ARTIFACTS)/geoip.dat
MIHOMO_GZ    := $(ARTIFACTS)/mihomo-linux-amd64.gz
MIHOMO_BIN	 := $(ARTIFACTS)/akasha-mihomo

META_RULES_BASE := https://github.com/MetaCubeX/meta-rules-dat/releases/download/latest

# Define color codes
GREEN  := \033[0;32m
RED    := \033[0;31m
BLUE   := \033[0;34m
YELLOW := \033[0;33m
# No Color (Reset)
NC     := \033[0m

.PHONY: prebuild clean mihomo-start run release

# ANCHOR: Project debug commands
mihomo-start:
	mihomo -f config/config.yaml -ext-ctl-unix /tmp/akasha/mihomo.sock

run:
	cargo run

release:
	cargo build --release
# ANCHOR_END: Project debug commands

test:
	@echo "$(BLUE)[INFO]$(NC): Creating dir [\`$@\`]"

# ANCHOR: [`prebuild`] target
prebuild: \
		$(COUNTRY_MMDB) \
		$(GEOSITE_DAT) \
		$(GEOIP_DAT) \
		$(MIHOMO_BIN)
		@echo "$(GREEN)[SUCCESS]:$(NC) Finishing prebuild process!"

$(ARTIFACTS):
	@echo "$(BLUE)[INFO]:$(NC) Creating dir [\`$@\`]"
	@mkdir -p $(ARTIFACTS)

$(COUNTRY_MMDB): | $(ARTIFACTS)
	@echo "$(BLUE)[INFO]:$(NC) Downloading $(META_RULES_BASE)/country.mmdb"
	@curl -L \
		-o $@ \
		$(META_RULES_BASE)/country.mmdb

$(GEOSITE_DAT): | $(ARTIFACTS)
	@echo "$(BLUE)[INFO]:$(NC) Downloading $(META_RULES_BASE)/geosite.dat"
	@curl -L \
		-o $@ \
		$(META_RULES_BASE)/geosite.dat

$(GEOIP_DAT): | $(ARTIFACTS)
	@echo "$(BLUE)[INFO]:$(NC) Downloading $(META_RULES_BASE)/geoip.dat"
	@curl -L \
		-o $@ \
		$(META_RULES_BASE)/geoip.dat

$(MIHOMO_GZ): | $(ARTIFACTS)
	@url=$$( \
		curl -s https://api.github.com/repos/MetaCubeX/mihomo/releases/latest | \
		grep browser_download_url | \
		grep 'mihomo-linux-amd64-v2-v.*\.gz"' | \
		cut -d '"' -f 4 \
	); \
	echo "$(BLUE)[INFO]:$(NC) Downloading $$url"; \
	curl -fL -o $@ "$$url"

$(MIHOMO_BIN): $(MIHOMO_GZ)
	@echo "$(BLUE)[INFO]:$(NC) Extracting $@"
	@gzip -dc $< > $@
	@chmod +x $@
# ANCHOR_END: [`prebuild`] target

clean:
	rm -rf $(ARTIFACTS)
