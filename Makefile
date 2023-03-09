run:
	docker-compose \
		-p hollywood \
		-f docker/docker-compose.yml up \
		--build \
		--remove-orphans

stop:
	docker-compose \
		-f docker/docker-compose.yml down \
		--rmi=local \
		--volumes \
		--remove-orphans

shell:
	docker exec -it `docker ps --filter "name=hollywood_hollywood" -q` /bin/bash