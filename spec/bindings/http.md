# Shapes Specification: HTTP Transport Binding

**Version:** 0.1.0
**Status:** Draft

## Introduction

This document maps the abstract operations defined in
[operations.md](../operations.md) to HTTP endpoints. Each spec operation
maps to exactly one HTTP route. All request and response bodies use JSON.

For each endpoint, this document specifies the HTTP method, path, parameters,
response schema, and error responses. Implementations of the HTTP binding
MUST conform to the interface described here.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

---

## Conventions

### Base Path

All endpoints are relative to the server's base URL. An implementation MAY
serve the API at any path prefix (e.g., `https://api.example.com/v1`), but
the paths specified in this document MUST be appended to that prefix without
modification.

### Content Type

All responses MUST use `Content-Type: application/json`.

Write operations (`init`, `create`) accept `Content-Type: application/json`
request bodies. Servers MUST reject request bodies with unsupported content
types with `415 Unsupported Media Type`.

### Common Types

The following types are referenced throughout the endpoint specifications.

**NodeType:**

Valid values: `shape`, `constraint`, `amendment`, `profile`.

When used as a path segment or query parameter, the value is a plain string.

**NodeId:**

An integer (>= 0) or a non-empty string. When used as a path segment or
query parameter, the value is passed as a string. The server MUST parse
numeric strings as integers (e.g., path segment `5` is interpreted as
integer ID 5).

### Error Responses

When a request fails, the server MUST return an appropriate HTTP status code
and a JSON error body with the following structure:

```json
{
  "error": {
    "code": "string",
    "message": "string"
  }
}
```

- `code` (string, REQUIRED) -- Machine-readable error code.
- `message` (string, REQUIRED) -- Human-readable description of the failure.

**Standard error codes and their HTTP status mappings:**

| Error Code | HTTP Status | Description |
|------------|-------------|-------------|
| NotFound | `404 Not Found` | The requested node or graph does not exist. |
| ValidationError | `422 Unprocessable Entity` | A write operation would violate one or more graph invariants. |
| SchemaError | `400 Bad Request` | The provided data does not conform to the expected JSON Schema. |
| AlreadyExists | `409 Conflict` | The resource already exists (e.g., graph already initialized). |

---

## Endpoints

### GET /.well-known/shapes

**Abstract operation:** `discover`

**Description:** Check whether a Shapes graph exists and return summary
metadata including the spec version and node counts. This endpoint
follows the well-known URI convention defined in
[RFC 8615](https://www.rfc-editor.org/rfc/rfc8615) and is also specified
in [discovery.md](../discovery.md) Mechanism 3.

**Parameters:** None.

**Response:**

Status: `200 OK`

```json
{
  "type": "object",
  "required": ["version", "node_counts"],
  "additionalProperties": false,
  "properties": {
    "version": {
      "type": "string",
      "description": "Shapes Specification version of the graph."
    },
    "node_counts": {
      "type": "object",
      "required": ["shape", "constraint", "amendment", "profile"],
      "additionalProperties": false,
      "properties": {
        "shape": { "type": "integer", "minimum": 0 },
        "constraint": { "type": "integer", "minimum": 0 },
        "amendment": { "type": "integer", "minimum": 0 },
        "profile": { "type": "integer", "minimum": 0 }
      }
    }
  }
}
```

**Error responses:**

| Status | Description |
|--------|-------------|
| `404 Not Found` | No shapes graph is available at this host. |

---

### GET /shapes/{node_type}/{id}

**Abstract operation:** `get`

**Description:** Retrieve the full definition of a single node by type and
ID. Returns the complete node object including all fields.

**Path parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `node_type` | NodeType | Yes | The type of node to retrieve. |
| `id` | NodeId | Yes | The ID of the node to retrieve. |

**Example:** `GET /shapes/shape/5`

**Response:**

Status: `200 OK`

The full node object. The structure depends on `node_type` and conforms to
the JSON Schema for that node type (`shape.json`, `constraint.json`,
`amendment.json`, or `profile.json`).

**Error responses:**

| Status | Description |
|--------|-------------|
| `404 Not Found` | No node of the specified type with the specified ID exists. |

---

### GET /shapes

**Abstract operation:** `list`

**Description:** List nodes with optional filters. Returns a summary entry
for each matching node. Without filters, returns all nodes across all types.

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `node_type` | NodeType | No | Filter to a specific node type. Omit to list all types. |
| `status` | string | No | Filter by status name (e.g., `proposed`, `canonical`). |
| `kind` | string | No | Filter by kind. |

**Example:** `GET /shapes?node_type=shape&status=promoted`

**Response:**

Status: `200 OK`

```json
{
  "type": "array",
  "items": {
    "type": "object",
    "required": ["node_type", "id", "name", "status", "kind"],
    "additionalProperties": false,
    "properties": {
      "node_type": {
        "type": "string",
        "enum": ["shape", "constraint", "amendment", "profile"]
      },
      "id": {
        "oneOf": [
          { "type": "integer", "minimum": 0 },
          { "type": "string", "minLength": 1 }
        ]
      },
      "name": { "type": "string" },
      "status": { "type": "string" },
      "kind": { "type": "string" }
    }
  }
}
```

An empty array is returned when no nodes match the filters. This is not an
error.

**Error responses:** None beyond transport-level errors.

---

### GET /shapes/tree

**Abstract operation:** `tree`

**Description:** Render the composition hierarchy as a tree. Defaults to
the shape DAG with constraint references shown inline.

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `node_type` | `shape` or `constraint` | No | Which DAG to render. Defaults to `shape`. |
| `root` | NodeId | No | Show only the subtree rooted at this node. Omit to show all root nodes. |
| `depth` | integer | No | Maximum depth to render. Defaults to 10. |

**Example:** `GET /shapes/tree?node_type=shape&root=1&depth=3`

**Response:**

Status: `200 OK`

```json
{
  "type": "string",
  "description": "Textual tree rendering of the composition hierarchy."
}
```

The tree is returned as a JSON string value. Structured tree data is
available through the `get` and `query.descendants` endpoints.

**Error responses:**

| Status | Description |
|--------|-------------|
| `404 Not Found` | The specified root node does not exist. |

---

### GET /shapes/query/ancestors

**Abstract operation:** `query.ancestors`

**Description:** Walk up the parent chain of a node and return all ancestor
IDs in BFS order. The starting node's own ID is not included.

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `node_type` | `shape` or `constraint` | Yes | Which DAG to traverse. |
| `id` | NodeId | Yes | The starting node ID. |

**Example:** `GET /shapes/query/ancestors?node_type=shape&id=5`

**Response:**

Status: `200 OK`

```json
{
  "type": "array",
  "items": {
    "oneOf": [
      { "type": "integer", "minimum": 0 },
      { "type": "string", "minLength": 1 }
    ]
  },
  "description": "Ancestor NodeIds in BFS order."
}
```

**Error responses:**

| Status | Description |
|--------|-------------|
| `404 Not Found` | The specified node does not exist. |

---

### GET /shapes/query/descendants

**Abstract operation:** `query.descendants`

**Description:** Walk down the child tree of a node and return all
descendant IDs in BFS order. The starting node's own ID is not included.

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `node_type` | `shape` or `constraint` | Yes | Which DAG to traverse. |
| `id` | NodeId | Yes | The starting node ID. |

**Example:** `GET /shapes/query/descendants?node_type=shape&id=1`

**Response:**

Status: `200 OK`

```json
{
  "type": "array",
  "items": {
    "oneOf": [
      { "type": "integer", "minimum": 0 },
      { "type": "string", "minLength": 1 }
    ]
  },
  "description": "Descendant NodeIds in BFS order."
}
```

**Error responses:**

| Status | Description |
|--------|-------------|
| `404 Not Found` | The specified node does not exist. |

---

### GET /shapes/query/constraints

**Abstract operation:** `query.constraints`

**Description:** Compute the full set of effective constraints for a shape,
including constraints inherited from ancestor shapes. Each constraint is
annotated with its source shape and whether it was inherited.

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `shape_id` | NodeId | Yes | The shape ID to query constraints for. |

**Example:** `GET /shapes/query/constraints?shape_id=5`

**Response:**

Status: `200 OK`

```json
{
  "type": "array",
  "items": {
    "type": "object",
    "required": ["constraint_id", "constraint_name", "source_shape_id", "inherited"],
    "additionalProperties": false,
    "properties": {
      "constraint_id": {
        "oneOf": [
          { "type": "integer", "minimum": 0 },
          { "type": "string", "minLength": 1 }
        ],
        "description": "The ID of the constraint node."
      },
      "constraint_name": {
        "type": "string",
        "description": "The name of the constraint node."
      },
      "source_shape_id": {
        "oneOf": [
          { "type": "integer", "minimum": 0 },
          { "type": "string", "minLength": 1 }
        ],
        "description": "The ID of the shape that references this constraint."
      },
      "inherited": {
        "type": "boolean",
        "description": "False if the constraint is directly referenced by the queried shape, true if inherited from an ancestor."
      }
    }
  }
}
```

**Error responses:**

| Status | Description |
|--------|-------------|
| `404 Not Found` | The specified shape does not exist. |

---

### GET /shapes/validate

**Abstract operation:** `validate`

**Description:** Check the entire graph against all defined invariants
(INV-001 through INV-011). Returns all violations found. An empty array
means the graph is valid.

**Parameters:** None.

**Response:**

Status: `200 OK`

```json
{
  "type": "array",
  "items": {
    "type": "object",
    "required": ["severity", "node_type", "node_id", "message"],
    "additionalProperties": false,
    "properties": {
      "severity": {
        "type": "string",
        "enum": ["error", "warning"],
        "description": "Severity of the invariant violation."
      },
      "node_type": {
        "type": "string",
        "description": "The node type where the violation was detected."
      },
      "node_id": {
        "type": "string",
        "description": "The ID of the node where the violation was detected."
      },
      "message": {
        "type": "string",
        "description": "Human-readable description of the invariant violation."
      }
    }
  }
}
```

Invariant violations are returned as data, not as HTTP errors. A `200 OK`
response with a non-empty array indicates the graph has violations.

**Error responses:** None beyond transport-level errors.

---

### POST /shapes/init

**Abstract operation:** `init`

**Description:** Initialize a new, empty Shapes graph. Creates the storage
structure and returns the version and location of the created graph.

**Request body:**

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "path": {
      "type": "string",
      "description": "Directory in which to initialize. Defaults to the server's configured working directory."
    }
  }
}
```

The request body MAY be omitted or empty to use the server's default path.

**Response:**

Status: `201 Created`

```json
{
  "type": "object",
  "required": ["version", "path"],
  "additionalProperties": false,
  "properties": {
    "version": {
      "type": "string",
      "description": "Shapes Specification version of the created graph."
    },
    "path": {
      "type": "string",
      "description": "Path to the created graph root."
    }
  }
}
```

**Error responses:**

| Status | Description |
|--------|-------------|
| `409 Conflict` | A shapes graph already exists at the specified path. |

---

### POST /shapes

**Abstract operation:** `create`

**Description:** Create a new node and add it to the graph. The server
assigns the next available ID. The node starts in `proposed` status unless
the data specifies otherwise. After creation, the server verifies that the
graph remains valid.

**Request body:**

```json
{
  "type": "object",
  "required": ["node_type", "data"],
  "additionalProperties": false,
  "properties": {
    "node_type": {
      "type": "string",
      "enum": ["shape", "constraint", "amendment", "profile"],
      "description": "The type of node to create."
    },
    "data": {
      "type": "object",
      "description": "The node definition. The 'id' field, if present, is ignored; the server assigns the ID. All other fields follow the JSON Schema for the specified node type."
    }
  }
}
```

The `data` object MUST conform to the JSON Schema for the specified
`node_type`, with the exception of the `id` field which is server-assigned.

**Response:**

Status: `201 Created`

The full node object as stored, including the assigned ID. The structure
depends on `node_type` and conforms to the JSON Schema for that node type.

The response MUST include a `Location` header with the URL of the created
node (e.g., `Location: /shapes/shape/17`).

**Error responses:**

| Status | Description |
|--------|-------------|
| `400 Bad Request` | The provided data does not conform to the JSON Schema for the specified node type. |
| `422 Unprocessable Entity` | The new node would violate one or more graph invariants. |

---

## Endpoint Summary

| Method | Endpoint | Abstract Operation | Parameters | Returns |
|--------|----------|--------------------|------------|---------|
| `GET` | `/.well-known/shapes` | discover | -- | `{version, node_counts}` |
| `GET` | `/shapes/{node_type}/{id}` | get | Path: `node_type`, `id` | Full node object |
| `GET` | `/shapes` | list | Query: `node_type?`, `status?`, `kind?` | `[{node_type, id, name, status, kind}]` |
| `GET` | `/shapes/tree` | tree | Query: `node_type?`, `root?`, `depth?` | Textual tree rendering (string) |
| `GET` | `/shapes/query/ancestors` | query.ancestors | Query: `node_type`, `id` | `[NodeId]` |
| `GET` | `/shapes/query/descendants` | query.descendants | Query: `node_type`, `id` | `[NodeId]` |
| `GET` | `/shapes/query/constraints` | query.constraints | Query: `shape_id` | `[{constraint_id, constraint_name, source_shape_id, inherited}]` |
| `GET` | `/shapes/validate` | validate | -- | `[{severity, node_type, node_id, message}]` |
| `POST` | `/shapes/init` | init | Body: `{path?}` | `{version, path}` |
| `POST` | `/shapes` | create | Body: `{node_type, data}` | Full node object |
