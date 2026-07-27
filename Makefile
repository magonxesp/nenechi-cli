.PHONY: build-docker-image publish-docker-image deb deb-debug

build-docker-image:
	echo "$$(date +%Y-%m-%d.%H%M%S)" > .version \
	&& export NENECHI_CLI_DOCKER_IMAGE="magonx/nenechi-cli:$$(cat .version)" \
	&& docker build . -t "$$NENECHI_CLI_DOCKER_IMAGE" \
	&& echo "Docker image: $$NENECHI_CLI_DOCKER_IMAGE"

publish-docker-image:
	docker push "magonx/nenechi-cli:$$(cat .version)"

deb:
	cargo clean \
	&& cargo build --release \
	&& bash scripts/deb-package.sh release

deb-debug:
	cargo clean \
	&& cargo build \
	&& bash scripts/deb-package.sh debug
