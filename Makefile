# Bandit is a static code analysis tool to detect security vulnerabilities in Python applications
# https://wiki.openstack.org/wiki/Security/Projects/Bandit
.PHONY: all
all: help

.PHONY: lint
lint: rust-lint check-isort check-black ## Run Rust and Python linters/formatters

.PHONY: black
black: ## Run black for file formatting
	@echo "Running black (may result in changes in your working directory)…"
	@poetry run black securedrop_proxy tests

.PHONY: check-black
check-black: ## Check Python source code formatting with black
	@echo "Running black formatting check…"
	@poetry run black --check --diff securedrop_proxy tests

.PHONY: isort
isort: ## Run isort for file formatting
	@echo "Running isort (may result in changes in your working directory)…"
	@poetry run isort securedrop_proxy/*.py tests/*.py

.PHONY: check-isort
check-isort: ## Check isort for file formatting
	@echo "Running isort module ordering check…"
	@poetry run isort --check-only --diff securedrop_proxy/*.py tests/*.py

.PHONY: test
test: ## Runs integration tests
	@cargo build
	@poetry run pytest

.PHONY: check
check: lint rust-test test  ## Runs all tests and code checkers

.PHONY: rust-lint
rust-lint: ## Lint Rust code
	@echo "Linting Rust code..."
	cargo fmt --check
	cargo clippy

.PHONY: rust-test
rust-test: ## Run Rust tests
	@echo "Running Rust tests..."
	cargo test

.PHONY: rust-audit
rust-audit: ## check dependencies in Cargo.lock
	@echo "███ Running Rust dependency checks..."
	@cargo install cargo-audit
	@cargo audit
	@echo

# Explanation of the below shell command should it ever break.
# 1. Set the field separator to ": ##" and any make targets that might appear between : and ##
# 2. Use sed-like syntax to remove the make targets
# 3. Format the split fields into $$1) the target name (in blue) and $$2) the target descrption
# 4. Pass this file as an arg to awk
# 5. Sort it alphabetically
# 6. Format columns with colon as delimiter.
.PHONY: help
help: ## Print this message and exit.
	@printf "Makefile for developing and testing the SecureDrop proxy.\n"
	@printf "Subcommands:\n\n"
	@awk 'BEGIN {FS = ":.*?## "} /^[0-9a-zA-Z_-]+:.*?## / {printf "\033[36m%s\033[0m : %s\n", $$1, $$2}' $(MAKEFILE_LIST) \
		| sort \
		| column -s ':' -t
