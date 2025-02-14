DEV_COMPOSE_FILE=docker-compose.dev.yml
PROD_COMPOSE_FILE=docker-compose.yml

dev:
	@echo " ____   ___       _    _     ____  ___  ____  ___ _____ _   _ __  __ "
	@echo "|  _ \ / _ \     / \  | |   / ___|/ _ \|  _ \|_ _|_   _| | | |  \/  |"
	@echo "| | | | | | |   / _ \ | |  | |  _| | | | |_| || |  | | | |_| | |\/| |"
	@echo "| |_| | |_| |  / ___ \| |__| |_| | |_| |  _ < | |  | | |  _  | |  | |"
	@echo "|____/ \__\_\ /_/   \_\_____\____|\___/|_| \_\___| |_| |_| |_|_|  |_|"
	@echo "                                                       by Tiiita     "
	@echo "Starting Docker Compose in development mode..."
	docker-compose -f $(DEV_COMPOSE_FILE) up --build

prod:
	@echo " ____   ___       _    _     ____  ___  ____  ___ _____ _   _ __  __ "
	@echo "|  _ \ / _ \     / \  | |   / ___|/ _ \|  _ \|_ _|_   _| | | |  \/  |"
	@echo "| | | | | | |   / _ \ | |  | |  _| | | | |_| || |  | | | |_| | |\/| |"
	@echo "| |_| | |_| |  / ___ \| |__| |_| | |_| |  _ < | |  | | |  _  | |  | |"
	@echo "|____/ \__\_\ /_/   \_\_____\____|\___/|_| \_\___| |_| |_| |_|_|  |_|"
	@echo "                                                       by Tiiita     "
	@echo "Starting Docker Compose in production mode..."
	docker-compose -f $(PROD_COMPOSE_FILE) up --build

down:
	@echo "Stopping and removing containers..."
	docker-compose -f $(DEV_COMPOSE_FILE) down
	docker-compose -f $(PROD_COMPOSE_FILE) down

build:
	@echo " ____   ___       _    _     ____  ___  ____  ___ _____ _   _ __  __ "
	@echo "|  _ \ / _ \     / \  | |   / ___|/ _ \|  _ \|_ _|_   _| | | |  \/  |"
	@echo "| | | | | | |   / _ \ | |  | |  _| | | | |_| || |  | | | |_| | |\/| |"
	@echo "| |_| | |_| |  / ___ \| |__| |_| | |_| |  _ < | |  | | |  _  | |  | |"
	@echo "|____/ \__\_\ /_/   \_\_____\____|\___/|_| \_\___| |_| |_| |_|_|  |_|"
	@echo "                                                       by Tiiita     "
	@echo "Building Docker images..."
	docker-compose -f $(DEV_COMPOSE_FILE) build
	docker-compose -f $(PROD_COMPOSE_FILE) build

logs:
	@echo "Tailing logs of containers..."
	docker-compose -f $(DEV_COMPOSE_FILE) logs -f