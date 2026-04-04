# Shapes Specification: MCP Server Transport Binding

**Version:** 0.1.0
**Status:** Draft

## Introduction

This document maps the abstract operations defined in
[operations.md](../operations.md) to MCP (Model Context Protocol) tools.
Each spec operation maps to exactly one MCP tool. All tools accept JSON
parameters and return JSON responses.

For each tool, this document specifies the tool name, description, input
schema, output schema, and error responses. Implementations of the MCP
binding MUST conform to the interface described here.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

---

## Conventions

### Tool Naming

All tool names use the prefix `shapes_` followed by the operation name with
underscores separating words. For operations in the `query` namespace, the
dot is replaced with an underscore (e.g., `query.ancestors` becomes
`shapes_query_ancestors`).

### Common Types

The following JSON Schema definitions are referenced throughout the tool
specifications.

**NodeType:**

```json
{
  "type": "string",
  "enum": ["shape", "constraint", "amendment", "profile"]
}
```

**NodeId:**

```json
{
  "oneOf": [
    { "type": "integer", "minimum": 0 },
    { "type": "string", "minLength": 1 }
  ]
}
```

### Error Responses

When a tool call fails, the server MUST return an MCP error response. Error
responses use the standard MCP `isError: true` content format with a text
content block describing the failure.

| Error Condition | Description |
|-----------------|-------------|
| NotFound | The requested node or graph does not exist. |
| ValidationError | A write operation would violate one or more graph invariants. |
| SchemaError | The provided data does not conform to the expected JSON Schema. |

---

## Tools

### shapes_discover

**Abstract operation:** `discover`

**Description:** Check whether a Shapes graph exists and return summary
metadata including the spec version and node counts.

**Input Schema:**

```json
{
  "type": "object",
  "properties": {},
  "additionalProperties": false
}
```

No parameters. The server determines the graph location from its
configuration.

**Output Schema:**

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

| Condition | Description |
|-----------|-------------|
| NotFound | No shapes graph is available to the server. |

---

### shapes_get

**Abstract operation:** `get`

**Description:** Retrieve the full definition of a single node by type and
ID. Returns the complete node object including all fields.

**Input Schema:**

```json
{
  "type": "object",
  "required": ["node_type", "id"],
  "additionalProperties": false,
  "properties": {
    "node_type": {
      "type": "string",
      "enum": ["shape", "constraint", "amendment", "profile"],
      "description": "The type of node to retrieve."
    },
    "id": {
      "oneOf": [
        { "type": "integer", "minimum": 0 },
        { "type": "string", "minLength": 1 }
      ],
      "description": "The ID of the node to retrieve."
    }
  }
}
```

**Output Schema:**

The full node object. The structure depends on `node_type` and conforms to
the JSON Schema for that node type (`shape.json`, `constraint.json`,
`amendment.json`, or `profile.json`).

**Error responses:**

| Condition | Description |
|-----------|-------------|
| NotFound | No node of the specified type with the specified ID exists. |

---

### shapes_list

**Abstract operation:** `list`

**Description:** List nodes with optional filters. Returns a summary entry
for each matching node. Without filters, returns all nodes across all types.

**Input Schema:**

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "node_type": {
      "type": "string",
      "enum": ["shape", "constraint", "amendment", "profile"],
      "description": "Filter to a specific node type. Omit to list all types."
    },
    "status": {
      "type": "string",
      "description": "Filter by status name (e.g., 'proposed', 'canonical')."
    },
    "kind": {
      "type": "string",
      "description": "Filter by kind."
    }
  }
}
```

**Output Schema:**

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

### shapes_tree

**Abstract operation:** `tree`

**Description:** Render the composition hierarchy as a tree. Defaults to
the shape DAG with constraint references shown inline.

**Input Schema:**

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "node_type": {
      "type": "string",
      "enum": ["shape", "constraint"],
      "description": "Which DAG to render. Defaults to 'shape'."
    },
    "root": {
      "oneOf": [
        { "type": "integer", "minimum": 0 },
        { "type": "string", "minLength": 1 }
      ],
      "description": "Show only the subtree rooted at this node. Omit to show all root nodes."
    },
    "depth": {
      "type": "integer",
      "minimum": 0,
      "description": "Maximum depth to render. Defaults to 10."
    }
  }
}
```

**Output Schema:**

```json
{
  "type": "string",
  "description": "Textual tree rendering of the composition hierarchy."
}
```

The tree is returned as a formatted text string. MCP consumers display the
string directly. Structured tree data is available through `shapes_get` and
`shapes_query_descendants`.

**Error responses:**

| Condition | Description |
|-----------|-------------|
| NotFound | The specified root node does not exist. |

---

### shapes_query_ancestors

**Abstract operation:** `query.ancestors`

**Description:** Walk up the parent chain of a node and return all ancestor
IDs in BFS order. The starting node's own ID is not included.

**Input Schema:**

```json
{
  "type": "object",
  "required": ["node_type", "id"],
  "additionalProperties": false,
  "properties": {
    "node_type": {
      "type": "string",
      "enum": ["shape", "constraint"],
      "description": "Which DAG to traverse. Must be 'shape' or 'constraint'."
    },
    "id": {
      "oneOf": [
        { "type": "integer", "minimum": 0 },
        { "type": "string", "minLength": 1 }
      ],
      "description": "The starting node ID."
    }
  }
}
```

**Output Schema:**

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

| Condition | Description |
|-----------|-------------|
| NotFound | The specified node does not exist. |

---

### shapes_query_descendants

**Abstract operation:** `query.descendants`

**Description:** Walk down the child tree of a node and return all
descendant IDs in BFS order. The starting node's own ID is not included.

**Input Schema:**

```json
{
  "type": "object",
  "required": ["node_type", "id"],
  "additionalProperties": false,
  "properties": {
    "node_type": {
      "type": "string",
      "enum": ["shape", "constraint"],
      "description": "Which DAG to traverse. Must be 'shape' or 'constraint'."
    },
    "id": {
      "oneOf": [
        { "type": "integer", "minimum": 0 },
        { "type": "string", "minLength": 1 }
      ],
      "description": "The starting node ID."
    }
  }
}
```

**Output Schema:**

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

| Condition | Description |
|-----------|-------------|
| NotFound | The specified node does not exist. |

---

### shapes_query_constraints

**Abstract operation:** `query.constraints`

**Description:** Compute the full set of effective constraints for a shape,
including constraints inherited from ancestor shapes. Each constraint is
annotated with its source shape and whether it was inherited.

**Input Schema:**

```json
{
  "type": "object",
  "required": ["shape_id"],
  "additionalProperties": false,
  "properties": {
    "shape_id": {
      "oneOf": [
        { "type": "integer", "minimum": 0 },
        { "type": "string", "minLength": 1 }
      ],
      "description": "The shape ID to query constraints for."
    }
  }
}
```

**Output Schema:**

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

| Condition | Description |
|-----------|-------------|
| NotFound | The specified shape does not exist. |

---

### shapes_validate

**Abstract operation:** `validate`

**Description:** Check the entire graph against all defined invariants
(INV-001 through INV-011). Returns all violations found. An empty array
means the graph is valid.

**Input Schema:**

```json
{
  "type": "object",
  "properties": {},
  "additionalProperties": false
}
```

No parameters.

**Output Schema:**

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

**Error responses:** None beyond transport-level errors. Invariant
violations are returned as data, not as errors.

---

### shapes_init

**Abstract operation:** `init`

**Description:** Initialize a new, empty Shapes graph. Creates the storage
structure and returns the version and path of the created graph.

**Input Schema:**

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

**Output Schema:**

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

| Condition | Description |
|-----------|-------------|
| AlreadyExists | A shapes graph already exists at the specified path. |

---

### shapes_create

**Abstract operation:** `create`

**Description:** Create a new node and add it to the graph. The server
assigns the next available ID. The node starts in `proposed` status unless
the data specifies otherwise. After creation, the server verifies that the
graph remains valid.

**Input Schema:**

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

**Output Schema:**

The full node object as stored, including the assigned ID. The structure
depends on `node_type` and conforms to the JSON Schema for that node type.

**Error responses:**

| Condition | Description |
|-----------|-------------|
| ValidationError | The new node would violate one or more graph invariants. |
| SchemaError | The provided data does not conform to the JSON Schema for the specified node type. |

---

## Tool Summary

| Tool | Abstract Operation | Parameters | Returns |
|------|-------------------|------------|---------|
| `shapes_discover` | discover | `{}` | `{version, node_counts}` |
| `shapes_get` | get | `{node_type, id}` | Full node object |
| `shapes_list` | list | `{node_type?, status?, kind?}` | `[{node_type, id, name, status, kind}]` |
| `shapes_tree` | tree | `{node_type?, root?, depth?}` | Textual tree rendering (string) |
| `shapes_query_ancestors` | query.ancestors | `{node_type, id}` | `[NodeId]` |
| `shapes_query_descendants` | query.descendants | `{node_type, id}` | `[NodeId]` |
| `shapes_query_constraints` | query.constraints | `{shape_id}` | `[{constraint_id, constraint_name, source_shape_id, inherited}]` |
| `shapes_validate` | validate | `{}` | `[{severity, node_type, node_id, message}]` |
| `shapes_init` | init | `{path?}` | `{version, path}` |
| `shapes_create` | create | `{node_type, data}` | Full node object |
