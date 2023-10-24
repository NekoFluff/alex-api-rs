# Local Development

Uses Docker Compose

There is a [Makefile](Makefile) with some useful commands. Run `make up` to start the containers. Run `make down-rmi` to stop the containers and remove the images.

## .env

The `docker-compose` file uses environment variables in the `.env` file. Copy the `.env.example` file to `.env` and fill in the values.

# DockerHub

## Building an Image

Run `make build` to build the image. This will tag the image as `nekofluff/alex-api-rs:latest`.

## Pushing an Image

Run `make push` to push the image to DockerHub.

# K8s Deployment

Uses Kubernetes

Look in [build/otel-collector/README.md](build/otel-collector/README.md) for more information.