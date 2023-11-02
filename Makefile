run:
	cargo run

up:
	docker compose --file=build/docker-compose.yml --env-file=.env up -d

down:
	docker compose --file=build/docker-compose.yml down

down-rmi:
	docker compose --file=build/docker-compose.yml down --rmi local

build-multi-arch:
	docker buildx create --name multi-arch \
	--platform "linux/amd64,linux/arm/v7" \
	--driver "docker-container"
	docker buildx use multi-arch

build-image:
	docker buildx build \
	--platform "linux/amd64,linux/arm/v7" \
	--tag NekoFluff/alex-api-rs:latest \
	--push . -f build/Dockerfile.build

push:
	docker push nekofluff/alex-api-rs:latest

deploy-up:
	docker image pull nekofluff/alex-api-rs
	docker compose --file=build/docker-compose.deploy.yml --env-file=.env up -d

deploy-down:
	docker compose --file=build/docker-compose.deploy.yml down