# Zenith Control Plane

REST API for managing Zenith Data Plane infrastructure.

## Features

- **Node Management**: Register/deregister data plane nodes
- **Plugin Registry**: Central plugin management
- **Deployment Orchestration**: Deploy plugins to nodes
- **Health Monitoring**: System health and metrics

## API Endpoints

### Health & Info
- `GET /health` - Health check
- `GET /api/v1/info` - System information

### Nodes
- `GET /api/v1/nodes` - List all nodes
- `POST /api/v1/nodes` - Register node
- `GET /api/v1/nodes/:id` - Get node details
- `DELETE /api/v1/nodes/:id` - Deregister node

### Plugins
- `GET /api/v1/plugins` - List plugins
- `POST /api/v1/plugins` - Register plugin
- `DELETE /api/v1/plugins/:id` - Delete plugin

### Deployments
- `GET /api/v1/deployments` - List deployments
- `POST /api/v1/deployments` - Create deployment
- `DELETE /api/v1/deployments/:id` - Delete deployment

## Running

```bash
cd control-plane
cargo run --release
```

Server starts on `http://0.0.0.0:9090`

## API Examples

### Register a Node
```bash
curl -X POST http://localhost:9090/api/v1/nodes \
  -H "Content-Type: application/json" \
  -d '{"address": "10.0.0.1:8080", "capacity": 1000000}'
```

### Register a Plugin
```bash
curl -X POST http://localhost:9090/api/v1/plugins \
  -H "Content-Type: application/json" \
  -d '{
    "name": "rate_limiter",
    "version": "1.0.0",
    "wasm_url": "https://example.com/plugins/rate_limiter.wasm"
  }'
```

### Create Deployment
```bash
curl -X POST http://localhost:9090/api/v1/deployments \
  -H "Content-Type: application/json" \
  -d '{
    "plugin_id": "<plugin-id>",
    "node_ids": ["<node-id-1>", "<node-id-2>"]
  }'
```

## Response Format

All successful responses return JSON:

```json
{
  "id": "uuid",
  "status": "active",
  ...
}
```

## Architecture

```
Control Plane (Port 9090)
├── Node Registry
├── Plugin Registry
└── Deployment Manager
```

Integrates with:
- Data Plane nodes
- Dashboard
- External management tools
