.PHONY: build-docker-image publish-docker-image

build-docker-image:
	echo "$$(date +%Y-%m-%d.%H%M%S)" > .version \
	&& export NENECHI_CLI_DOCKER_IMAGE="magonx/nenechi-cli:$$(cat .version)" \
	&& docker build . -t "$$NENECHI_CLI_DOCKER_IMAGE" \
	&& echo "Docker image: $$NENECHI_CLI_DOCKER_IMAGE"

publish-docker-image:
	docker push "magonx/nenechi-cli:$$(cat .version)"
