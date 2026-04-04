# Shapes Specification: Discovery

**Version:** 0.1.0
**Status:** Draft

## Introduction

This document specifies how consumers -- agents, tools, and integrations --
discover that a project uses the Shapes Specification and locate its
shapes graph. Discovery is the prerequisite for all other spec
operations: before an agent can read, query, or mutate a shapes graph, it
must determine whether one exists and how to access it.

Three discovery mechanisms are defined. An implementation MUST support at
least one mechanism. Consumers SHOULD attempt mechanisms in the order
listed (filesystem, MCP, HTTP) and use the first that succeeds.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

---

## Mechanism 1: Filesystem Discovery

Filesystem discovery is the primary mechanism for local tooling and CLI-based
agents. It relies on a conventional directory layout with no external
dependencies.

### Convention

A Shapes graph is stored in a `.shapes/` directory at the project root or in
a nearest ancestor directory of the current working directory.

### Directory Structure

The `.shapes/` directory MUST contain:

- **`meta.yaml`** -- Graph metadata conforming to the
  [Meta JSON Schema](schema/meta.json). Contains the spec version and
  next-ID counters for each node type.

- **`shapes/`** -- Directory containing Shape node files.

- **`constraints/`** -- Directory containing Constraint node files.

- **`amendments/`** -- Directory containing Amendment node files.

- **`profiles/`** -- Directory containing Profile node files.

Node files within each subdirectory are named `{id}.yaml`, where `{id}` is
the node's numeric identifier (e.g., `shapes/5.yaml`, `constraints/3.yaml`).

### File Format

YAML is the canonical file format for filesystem storage. Each node file
MUST be a valid YAML document conforming to the JSON Schema for its node
type.

### Version Control

The `.shapes/` directory SHOULD be version-controlled (committed to the
project's git repository). Tracking shapes alongside source code ensures
that intent, constraints, and structural context evolve with the codebase.

### Discovery Algorithm

To discover a shapes graph from a given starting directory, implementations
MUST use the following walk-up algorithm:

1. Let `dir` be the starting directory (typically the current working
   directory).
2. Check whether `{dir}/.shapes/meta.yaml` exists and is a readable file.
3. If it exists, parse `meta.yaml`. If parsing succeeds and the file
   conforms to the Meta schema, discovery succeeds. The shapes graph root
   is `{dir}/.shapes/`.
4. If it does not exist, set `dir` to the parent directory of `dir`.
5. If `dir` is the filesystem root (no parent), discovery fails.
6. Repeat from step 2.

Implementations MUST NOT traverse above the filesystem root. Implementations
MAY impose a maximum traversal depth to avoid scanning unrelated directory
trees, but the default behavior SHOULD be to walk up until the filesystem
root is reached.

### Verification

After locating `.shapes/meta.yaml`, the implementation SHOULD verify that the
required subdirectories (`shapes/`, `constraints/`, `amendments/`,
`profiles/`) exist. Missing subdirectories indicate a corrupted or partially
initialized graph.

---

## Mechanism 2: MCP Capability Advertisement

When the Shapes Specification is exposed as an MCP (Model Context
Protocol) server, discovery happens through the MCP tool listing mechanism.

### Tool Advertisement

An MCP server implementing the Shapes Specification MUST advertise a tool
named `shapes_discover` in its tool list. The presence of any tools with the
`shapes_` prefix in the MCP tool list signals that the server supports the
Shapes Specification.

### Discovery Procedure

1. The consumer lists available MCP tools (via the standard MCP
   `tools/list` method).
2. If the tool list contains `shapes_discover`, the server supports Shapes.
3. The consumer calls `shapes_discover` with no parameters (empty object).
4. The server returns graph metadata.

### Response

Calling `shapes_discover` MUST return a JSON object with the following
structure:

```json
{
  "version": "0.1.0",
  "node_counts": {
    "shape": 16,
    "constraint": 11,
    "amendment": 0,
    "profile": 1
  }
}
```

- `version` (string, REQUIRED) -- The Shapes Specification version of
  the graph.
- `node_counts` (object, REQUIRED) -- The number of nodes of each type
  currently in the graph. MUST contain integer values for keys `shape`,
  `constraint`, `amendment`, and `profile`.

### Absence of Shapes Support

If the MCP tool list does not contain any `shapes_*` prefixed tools, the
server does not support the Shapes Specification. Consumers MUST treat
this as discovery failure and SHOULD fall back to another mechanism if
available.

---

## Mechanism 3: Well-Known HTTP Endpoint

For HTTP/REST implementations, discovery uses a well-known URI as defined
by [RFC 8615](https://www.rfc-editor.org/rfc/rfc8615).

### Endpoint

```
GET /.well-known/shapes
```

### Success Response

A successful response MUST have:

- **Status:** `200 OK`
- **Content-Type:** `application/json`
- **Body:** A JSON object with the following structure:

```json
{
  "version": "0.1.0",
  "node_counts": {
    "shape": 16,
    "constraint": 11,
    "amendment": 0,
    "profile": 1
  }
}
```

The fields carry the same meaning as in Mechanism 2:

- `version` (string, REQUIRED) -- The Shapes Specification version.
- `node_counts` (object, REQUIRED) -- Node counts per type, with integer
  values for `shape`, `constraint`, `amendment`, and `profile`.

### Absence of Shapes Support

If the server returns `404 Not Found` for `GET /.well-known/shapes`, no
shapes graph is available at that host. Consumers MUST treat a 404 as
discovery failure.

Implementations SHOULD return `404` (not `405` or `501`) when the shapes
endpoint is not configured, to maintain consistency with the well-known URI
convention.

### Error Responses

| Status | Meaning |
|--------|---------|
| 200    | Shapes graph exists. Body contains metadata. |
| 404    | No shapes graph at this host. |
| 500    | Server error. Consumer SHOULD retry or fall back. |

---

## Discovery Precedence

When a consumer has access to multiple discovery mechanisms (e.g., a local
filesystem and an MCP connection), the consumer SHOULD prefer the mechanism
most appropriate to its execution context:

- **Local CLI agents** SHOULD use filesystem discovery.
- **MCP-connected agents** SHOULD use MCP capability advertisement.
- **Remote HTTP clients** SHOULD use the well-known endpoint.

If a consumer attempts multiple mechanisms, it SHOULD try them in the order
listed in this document (filesystem, MCP, HTTP) and use the first that
succeeds.

A consumer MUST NOT combine metadata from multiple discovery sources. A
single discovered graph is the unit of interaction for all subsequent
operations.
