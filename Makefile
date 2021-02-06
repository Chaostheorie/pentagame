# Complete Makefile based of gitlab.com/C0balt/oxidized-cms wth slight modifications
# Allow command line args to be passed instead of serial execution of steps
# Based off https://stackoverflow.com/a/14061796
ifeq (db-migration,$(firstword $(MAKECMDGOALS)))
  RUN_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
  $(eval $(RUN_ARGS):;@:)
endif

db-setup:
	bash scripts/diesel.sh
db-docker-setup:
	bash scripts/diesel.sh docker-config.toml
.PHONY: db-migration
db-migration:
	bash scripts/diesel.sh migration $(RUN_ARGS)
db-print:
	bash scripts/diesel.sh print-schema
db-reset:
	bash scripts/diesel.sh reset
build:
	ASSET_ENVIRONMENT="PRODUCTION" cargo b --release
build-native:
	ASSET_ENVIRONMENT="PRODUCTION" RUSTFLAGS="-C target-cpu=native" cargo build --release
generate:
	ASSET_ENVIRONMENT="PRODUCTION" cargo b --release
	./target/release/pentagame generate
dev-generate:
	./target/debug/pentagame generate
dev-build:
	ASSET_ENVIRONMENT="DEVELOPMENT" cargo b -vv
dev-serve:
	ASSET_ENVIRONMENT="DEVELOPMENT" cargo b -vv
	./target/debug/pentagame serve
ci-build:
	cargo build --verbose
ci-test:
	cargo check --verbose
clean:
	rm -fr static/dist static/node_modules static/scss/.dir-changes
	cargo clean
package:
	# Create all packaged versions (at the moment requires cargo deb)
	bash scripts/package.sh
serve:
	./target/release/pentagame serve