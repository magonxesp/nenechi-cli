.PHONY: build-docker-image publish-docker-image deb deb-debug sandbox

build-docker-image:
	echo "$$(date +%Y-%m-%d.%H%M%S)" > .version \
	&& export NENECHI_CLI_DOCKER_IMAGE="magonx/nenechi-cli:$$(cat .version)" \
	&& docker build . -t "$$NENECHI_CLI_DOCKER_IMAGE" \
	&& echo "Docker image: $$NENECHI_CLI_DOCKER_IMAGE"

publish-docker-image:
	docker push "magonx/nenechi-cli:$$(cat .version)"

deb:
	cargo build --release \
	&& bash scripts/deb-package.sh release

deb-debug:
	cargo build \
	&& bash scripts/deb-package.sh debug

sandbox:
	mkdir -p sandbox/home/.config/nenechi/conf.d
	test -f sandbox/home/.config/nenechi/config.yaml \
		|| cp examples/config.yaml sandbox/home/.config/nenechi/config.yaml
	test -f sandbox/home/.config/nenechi/conf.d/jellyfin.yaml \
		|| cp examples/conf.d/jellyfin.yaml sandbox/home/.config/nenechi/conf.d/jellyfin.yaml
	test -f sandbox/home/.config/nenechi/conf.d/wallpapers.yaml \
		|| cp examples/conf.d/wallpapers.yaml sandbox/home/.config/nenechi/conf.d/wallpapers.yaml

install:
	dpkg --remove nenechi
	dpkg -i target/release/bundle/nenechi_*.deb
