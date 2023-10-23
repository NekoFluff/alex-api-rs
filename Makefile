run:
	cargo run

up:
	docker compose --file=docker-compose.yml --env-file=.env up -d

down:
	docker compose --file=docker-compose.yml down --rmi local

build:
	docker compose --file=docker-compose.build.yml build
	docker push nekofluff/alex-api-rs:latest

deploy-up:
	docker image pull nekofluff/alex-api-rs
	docker compose --file=docker-compose.deploy.yml --env-file=.env up -d

deploy-down:
	docker compose --file=docker-compose.deploy.yml down