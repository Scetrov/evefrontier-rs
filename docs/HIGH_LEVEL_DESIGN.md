# High-Level Design

Provide routing functionality for EVE Frontier through multiple interfaces:

- **Command Line Interface (CLI)**: Allow users to interact with route planning via terminal
  commands for quick access and automation.
- **AWS Lambda Functions**: Enable serverless execution of routing algorithms to handle requests
  without managing servers.
- **Kubernetes (K8s) Services**: Deploy scalable and resilient routing services within a Kubernetes
  cluster for high availability.
- **WASM Modules**: Integrate WebAssembly modules for efficient execution of routing logic in web
  environments.

## Functionality

Initially we aim to provide the following core functionalities:

- **Route Calculation**: Compute optimal routes between points in the EVE Frontier universe.
- **Scouting**: Compute near-optimal paths visiting all systems within range of a given start
  system.
- **Temperature Awareness**: Incorporate system temperature data into route calculations to avoid
  high-risk areas.
- **Fuel Awareness**: Factor in fuel consumption and refueling points when determining routes.

## Flexibility

We aim to provide optionality in the following areas to further understanding of problem solving
approaches in EVE Frontier:

- **Algorithims**: Support multiple routing algorithms to cater to different user needs and
  scenarios (Dijkstra, A\*, etc.).
- **Spatial Indexing Data Structures**: Utilize various spatial indexing techniques (Octrees,
  KD-trees, Voxel Grids, etc.) to optimize route calculations.
