.PHONY: help up status verify down clean cilium-gateway

help:
	@echo "Yubi WebAuthn Cluster Management"
	@echo ""
	@echo "Available targets:"
	@echo "  make up             - Run full automated setup (Minikube check, Cilium Gateway API, Rancher storage, CNPG operator, build & apply)"
	@echo "  make status         - Check status of StorageClasses, operators, database cluster, pods, and services"
	@echo "  make verify         - Run functional health assertions on Rancher storage, CNPG DB, Gateway API, and app deployments"
	@echo "  make cilium-gateway - Patch cilium-config in kube-system for Gateway API & restart Cilium workloads"
	@echo "  make down           - Remove Yubi application namespace and resources"
	@echo "  make clean          - Full cleanup including CNPG operator, Rancher provisioner, and build artifacts"

up:
	@chmod +x scripts/setup.sh
	@./scripts/setup.sh

cilium-gateway:
	@chmod +x scripts/cilium_gateway.sh
	@./scripts/cilium_gateway.sh

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

