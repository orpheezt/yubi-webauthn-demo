.PHONY: help up status verify down clean

help:
	@echo "Yubi WebAuthn Cluster Management"
	@echo ""
	@echo "Available targets:"
	@echo "  make up      - Run full automated setup (Minikube check, Rancher storage, CNPG operator, build & apply)"
	@echo "  make status  - Check status of StorageClasses, operators, database cluster, pods, and services"
	@echo "  make verify  - Run functional health assertions on Rancher storage, CNPG DB, and app deployments"
	@echo "  make down    - Remove Yubi application namespace and resources"
	@echo "  make clean   - Full cleanup including CNPG operator, Rancher provisioner, and build artifacts"

up:
	@chmod +x scripts/setup.sh
	@./scripts/setup.sh

status:
	@chmod +x scripts/status.sh
	@./scripts/status.sh

verify:
	@chmod +x scripts/verify.sh
	@./scripts/verify.sh

down:
	@chmod +x scripts/teardown.sh
	@./scripts/teardown.sh down

clean:
	@chmod +x scripts/teardown.sh
	@./scripts/teardown.sh clean
