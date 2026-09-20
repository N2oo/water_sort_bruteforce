# Every task of the monorepo, from its root.
#
#   make help        the list below
#   make dev         PostgreSQL, the API and the front end, together
#   make check       the tests and the type checks of both sides

BACKEND  := backend
FRONTEND := frontend
COMPOSE  := docker compose
CARGO    := cargo
NPM      := npm

# A level file for `make solve`; override it: make solve LEVEL=backend/levels/level146.json
LEVEL ?= $(BACKEND)/levels/level145.json

# Used by `make api` and `make migrate` when no DATABASE_URL is exported.
DATABASE_URL ?= postgres://water_sort:water_sort@localhost:5432/water_sort

.DEFAULT_GOAL := help

.PHONY: help
help: ## Show this help
	@grep -hE '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| sort \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}'

# --- install ----------------------------------------------------------------

.PHONY: install
install: install-frontend ## Install every dependency (cargo resolves on build)

.PHONY: install-frontend
install-frontend: ## Install the front end dependencies from the lockfile
	cd $(FRONTEND) && $(NPM) ci

# --- run --------------------------------------------------------------------

.PHONY: dev
dev: ## Run the whole stack in Docker (front end :5173, API :8080)
	$(COMPOSE) up --build

.PHONY: down
down: ## Stop the stack and drop its containers
	$(COMPOSE) down

.PHONY: clean-volumes
clean-volumes: ## Stop the stack and drop the database volume too
	$(COMPOSE) down -v

.PHONY: logs
logs: ## Follow the logs of the stack
	$(COMPOSE) logs -f

.PHONY: api
api: ## Run the API on the host, against a local PostgreSQL
	cd $(BACKEND) && DATABASE_URL=$(DATABASE_URL) $(CARGO) run -p water-sort-api

.PHONY: api-memory
api-memory: ## Run the API on the host with no database at all
	cd $(BACKEND) && STORAGE=memory $(CARGO) run -p water-sort-api

.PHONY: front
front: ## Run the front end on :5173, proxying /api to :8080
	cd $(FRONTEND) && $(NPM) run dev

.PHONY: solve
solve: ## Solve a level with the CLI: make solve LEVEL=backend/levels/level146.json
	cd $(BACKEND) && $(CARGO) run -p water-sort-cli -- $(patsubst $(BACKEND)/%,%,$(LEVEL))

.PHONY: migrate
migrate: ## Apply the database migrations as a separate step
	cd $(BACKEND) && DATABASE_URL=$(DATABASE_URL) $(CARGO) run -p water-sort-migration -- up

# --- quality ----------------------------------------------------------------
#
# `check` is what passes today. `fmt-check` and `lint-backend` do not: the
# original solver sources predate rustfmt and clippy being run on them, so they
# are kept in `check-strict` until that backlog is cleared.

.PHONY: check
check: test lint-frontend ## The tests of both sides, plus the front end types

.PHONY: check-strict
check-strict: fmt-check lint test ## …and rustfmt + clippy, which the historical sources still fail

.PHONY: fmt
fmt: ## Format the Rust sources
	cd $(BACKEND) && $(CARGO) fmt --all

.PHONY: fmt-check
fmt-check: ## Fail if the Rust sources are not formatted
	cd $(BACKEND) && $(CARGO) fmt --all -- --check

.PHONY: lint
lint: lint-backend lint-frontend ## Lint both sides

.PHONY: lint-backend
lint-backend: ## Clippy over the workspace, warnings denied
	cd $(BACKEND) && $(CARGO) clippy --workspace --all-targets -- -D warnings

.PHONY: lint-frontend
lint-frontend: ## Type check the front end (tsc -b)
	cd $(FRONTEND) && $(NPM) run lint

.PHONY: test
test: test-backend test-frontend ## Test both sides

.PHONY: test-backend
test-backend: ## Rules, domain and HTTP tests (add TEST_DATABASE_URL for the SQL ones)
	cd $(BACKEND) && $(CARGO) test --workspace

.PHONY: test-frontend
test-frontend: ## The rules mirrored in TypeScript (vitest)
	cd $(FRONTEND) && $(NPM) test

# --- build ------------------------------------------------------------------

.PHONY: build
build: build-backend build-frontend ## Build both sides for release

.PHONY: build-backend
build-backend: ## Build the Rust workspace in release mode
	cd $(BACKEND) && $(CARGO) build --release --workspace

.PHONY: build-frontend
build-frontend: ## Build the front end into frontend/dist
	cd $(FRONTEND) && $(NPM) run build

.PHONY: docker-build
docker-build: ## Build both images without starting anything
	$(COMPOSE) build

.PHONY: clean
clean: ## Drop the build outputs of both sides
	cd $(BACKEND) && $(CARGO) clean
	rm -rf $(FRONTEND)/dist
