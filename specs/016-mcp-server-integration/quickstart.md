# Quickstart: EVE Frontier MCP Server

This guide shows how to configure AI assistants to use the EVE Frontier MCP server.

## Prerequisites

1. **Install the EVE Frontier CLI** (includes MCP server):
   ```bash
   # From source
   cargo install --path crates/evefrontier-cli

   # Or download pre-built binary from releases
   ```

2. **Download the dataset** (automatic on first use):
   ```bash
   evefrontier-cli download
   ```

3. **Verify installation**:
   ```bash
   evefrontier-cli --version
   evefrontier-cli route Nod Brana  # Test routing works
   ```

## Claude Desktop Configuration

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or
`%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "evefrontier": {
      "command": "evefrontier-cli",
      "args": ["mcp"],
      "env": {
        "RUST_LOG": "warn"
      }
    }
  }
}
```

### With Custom Data Directory

```json
{
  "mcpServers": {
    "evefrontier": {
      "command": "evefrontier-cli",
      "args": ["mcp", "--data-dir", "/path/to/datasets"],
      "env": {
        "RUST_LOG": "warn"
      }
    }
  }
}
```

## VS Code GitHub Copilot Configuration

Add to `.vscode/mcp.json` in your workspace:

```json
{
  "servers": {
    "evefrontier": {
      "type": "stdio",
      "command": "evefrontier-cli",
      "args": ["mcp"]
    }
  }
}
```

Or in user settings (`~/.vscode/mcp.json`):

```json
{
  "servers": {
    "evefrontier": {
      "type": "stdio",
      "command": "evefrontier-cli",
      "args": ["mcp"]
    }
  }
}
```

## Cursor Configuration

Add to Cursor's MCP settings:

```json
{
  "mcpServers": {
    "evefrontier": {
      "command": "evefrontier-cli",
      "args": ["mcp"]
    }
  }
}
```

## Docker Deployment

For containerized environments:

```bash
# Build the MCP server image
docker build -t evefrontier-mcp -f crates/evefrontier-mcp/Dockerfile .

# Run (stdio mode for local use)
docker run -i evefrontier-mcp
```

Claude Desktop with Docker:

```json
{
  "mcpServers": {
    "evefrontier": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "evefrontier-mcp"]
    }
  }
}
```

## Testing the Connection

### Using MCP Inspector

```bash
# Install MCP Inspector
npm install -g @modelcontextprotocol/inspector

# Test the server
npx @modelcontextprotocol/inspector evefrontier-cli mcp
```

### Manual Testing

```bash
# Start the server and send an initialize request
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | evefrontier-cli mcp
```

## Available Tools

Once connected, you can ask your AI assistant:

### Route Planning
- "Find a route from Nod to Brana"
- "What's the shortest path from H:2L2S to Y:3R7E avoiding systems above 400K?"
- "Plan a gate-only route between D:2NAS and J:35IA"

### System Information
- "Tell me about the Nod system"
- "What's the temperature of Brana?"
- "How many planets are in system G:3OA0?"

### Spatial Queries
- "What systems are within 50 light-years of Nod?"
- "Find nearby cold systems (under 300K) within 30 ly of Brana"

### Gate Connections
- "What systems are connected to Nod by jump gates?"
- "List all gate connections from H:2L2S"

## Troubleshooting

### Server doesn't start

1. Check the CLI is in PATH:
   ```bash
   which evefrontier-cli
   ```

2. Verify dataset is downloaded:
   ```bash
   evefrontier-cli download
   ```

3. Check for errors with verbose logging:
   ```bash
   RUST_LOG=debug evefrontier-cli mcp
   ```

### "Unknown system" errors

The server uses fuzzy matching. If you get suggestions, try the suggested name:
```
Error: Unknown system 'Nodd'. Did you mean: Nod?
```

### Connection timeouts

The server may take a few seconds to load the dataset on first request. This is normal for cold
starts.

### Memory issues

The spatial index requires ~100MB of memory. If running in a constrained environment, disable
spatial queries:
```bash
evefrontier-cli mcp --no-spatial-index
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level (error, warn, info, debug, trace) | `warn` |
| `EVEFRONTIER_DATA_DIR` | Override dataset directory | OS cache dir |

## Next Steps

- Read the [full documentation](../../docs/MCP_SERVER.md) for advanced usage
- Check the [API contracts](./contracts/) for detailed schema information
- Report issues at https://github.com/scetrov/evefrontier-rs/issues
