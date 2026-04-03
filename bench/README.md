# SLOP Benchmark for Shapes

Benchmarks shapes-enhanced Claude Code against the [SLOP Code Bench](https://github.com/SprocketLab/slop-code-bench)
to measure how shapes-first development prevents code quality degradation across iterative changes.

## Prerequisites

- [uv](https://astral.sh/uv) installed (`curl -LsSf https://astral.sh/uv/install.sh | sh`)
- SLOP benchmark cloned and synced:
  ```bash
  git clone https://github.com/SprocketLab/slop-code-bench.git
  cd slop-code-bench && uv python pin 3.12 && uv sync
  ```
- Claude Code CLI installed (`claude --version`)
- Shapes CLI installed (`shapes --help`)
- A valid Claude OAuth token or API key

## Setup

Set `SHAPES_ROOT` to the root of your shapes checkout. The environment config
uses this to find skills and assets:

```bash
export SHAPES_ROOT=/path/to/shapes   # root of this repo
export SLOP_ROOT=/path/to/slop-code-bench
```

## Directory Structure

```
bench/
├── README.md                     # This file
├── configs/
│   ├── prompts/
│   │   └── shapes-context.jinja  # Jinja2 prompt template (delegates to skills)
│   ├── agents/
│   │   └── shapes-claude.yaml    # Agent config (unlimited steps/cost)
│   ├── environments/
│   │   └── local-py-shapes.yaml  # Environment that injects skills into workspace
│   └── assets/
│       └── CLAUDE.md             # Workspace CLAUDE.md (shapes-first instructions)
└── runs/                         # Archived results (one dir per run)
```

## Running

### Baseline (vanilla Claude Code, no shapes)

```bash
cd "$SLOP_ROOT"

ANTHROPIC_API_KEY="<your-key>" \
CLAUDE_CODE_OAUTH_TOKEN="<your-token>" \
uv run slop-code run \
  --agent claude_code \
  --model anthropic/opus-4.6 \
  --prompt just-solve \
  --environment local-py \
  --problem file_backup \
  thinking=high
```

### Shapes-enhanced

```bash
cd "$SLOP_ROOT"

SHAPES_ROOT="/path/to/shapes" \
ANTHROPIC_API_KEY="<your-token>" \
CLAUDE_CODE_OAUTH_TOKEN="<your-token>" \
uv run slop-code run \
  --agent "$SHAPES_ROOT/bench/configs/agents/shapes-claude.yaml" \
  --prompt "$SHAPES_ROOT/bench/configs/prompts/shapes-context.jinja" \
  --environment "$SHAPES_ROOT/bench/configs/environments/local-py-shapes.yaml" \
  --model claude_code_oauth/opus-4.6 \
  --problem file_backup \
  thinking=high
```

Set both `ANTHROPIC_API_KEY` and `CLAUDE_CODE_OAUTH_TOKEN` to your token value.
The benchmark reads from `ANTHROPIC_API_KEY`; Claude Code reads from `CLAUDE_CODE_OAUTH_TOKEN`.

### Running on multiple problems

Replace `--problem file_backup` with multiple `--problem` flags or omit to run all 20:

```bash
--problem file_backup --problem etl_pipeline --problem log_query
```

### Available problems

```
circuit_eval, code_search, dag_execution, database_migration, dynamic_buffer,
dynamic_config_service_api, etl_pipeline, eve_industry, eve_jump_planner,
eve_market_tools, eve_route_planner, execution_server, file_backup, file_merger,
file_query_tool, layered_config_synthesizer, log_query, metric_transform_lang,
migrate_configs, trajectory_api
```

## Archiving Runs

After a run completes, archive it:

```bash
RUN_NAME="shapes-v4"
RUN_DIR=$(ls -d "$SLOP_ROOT"/outputs/opus-4.6/claude_code-2.0.51_shapes-context_high_* | tail -1)

mkdir -p "$SHAPES_ROOT/bench/runs/$RUN_NAME"
cp -r "$RUN_DIR" "$SHAPES_ROOT/bench/runs/$RUN_NAME/output"
cp "$SHAPES_ROOT/bench/configs/prompts/shapes-context.jinja" "$SHAPES_ROOT/bench/runs/$RUN_NAME/prompt-snapshot.jinja"
cp -r "$SHAPES_ROOT/skills" "$SHAPES_ROOT/bench/runs/$RUN_NAME/skills-snapshot"
```

## Evaluating Results

```bash
# Per-checkpoint results
for cp in 1 2 3 4; do
  echo "=== CP$cp ==="
  python3 -c "
import json
with open('$RUN_DIR/file_backup/checkpoint_$cp/evaluation/report.json') as f:
    e = json.load(f)
with open('$RUN_DIR/file_backup/checkpoint_$cp/quality_analysis/overall_quality.json') as f:
    q = json.load(f)
print(f'  tests={e[\"summary\"][\"passed\"]}/{e[\"summary\"][\"total\"]}')
print(f'  cc_max={q[\"complexity\"][\"cc_max\"]}  conc={q[\"functions\"][\"cc_concentration\"]:.3f}  depth={q[\"functions\"][\"depth_max\"]}  funcs={q[\"functions\"][\"count\"]}')
"
done
```

## How It Works

The benchmark runs Claude Code on iterative coding problems (4-8 checkpoints each).
The shapes-enhanced setup:

1. **Environment** (`local-py-shapes.yaml`) copies shapes skills into the workspace
   as `.claude/commands/` and places a `CLAUDE.md` at workspace root. Uses
   `$SHAPES_ROOT` to locate files.
2. **Prompt template** (`shapes-context.jinja`) tells Claude to use shapes-first:
   - CP1: Bootstrap with `shapes-init` skill, create constraints, then implement
   - CP2+: Use `shapes-context` to read graph first, plan changes, then implement
   - Always: Use `shapes-maintain` to sync graph after code changes
3. **Skills** auto-trigger and guide Claude through the shapes-first workflow.

## Key Metrics

- **CC max**: Peak cyclomatic complexity of any function (lower = better)
- **CC concentration**: Fraction of complexity mass in high-CC functions (lower = better, erosion metric)
- **Depth**: Maximum nesting depth of any function (lower = better)
- **Tests**: Number of test cases passed (higher = better)

## Results Summary (file_backup problem)

| Metric (CP4) | Baseline | Shapes (avg of 3) | Improvement |
|---|---|---|---|
| CC max | 60 | 29 | -52% |
| CC concentration | .665 | .530 | -20% |
| Max nesting depth | 7 | 4.0 | -43% |
| Tests passed | 64/89 | 64/89 | Same |
