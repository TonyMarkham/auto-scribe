# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Auto-scribe is a Rust workspace project (Edition 2024). The workspace is currently empty with no crate members defined.

## Project Management

This repository uses the `/pm` skill for agile project management through the Blazor Agile Board CLI.

**P1 Project GUID:** `63aca871-3284-4bd4-9b52-54e747139f78`

### When to Use /pm

**Proactively create work items for:**
- New features or significant enhancements
- Bug fixes that require investigation or multi-step work
- Refactoring tasks
- Documentation or infrastructure work
- Any work that would benefit from tracking and organization

**Update work item status as you work:**
- Set to `in_progress` when starting work
- Add comments to capture decisions, blockers, or progress notes
- Update to `done` when completed
- Use `blocked` status if waiting on external dependencies

**Track time on substantial work:**
- Start a timer when beginning focused work on a task
- Stop the timer when switching tasks or completing work
- This provides visibility into effort and helps with planning

### Quick Commands

```bash
# List all work items in P1 project
.pm/bin/pm work-item list 63aca871-3284-4bd4-9b52-54e747139f78 --pretty

# Create a task
.pm/bin/pm work-item create \
  --project-id 63aca871-3284-4bd4-9b52-54e747139f78 \
  --type task \
  --title "Task title" \
  --description "Detailed description" \
  --priority high \
  --pretty

# Update task status
.pm/bin/pm work-item update <work-item-id> \
  --version <current-version> \
  --status in_progress \
  --pretty

# Add a comment
.pm/bin/pm comment create \
  --work-item-id <work-item-id> \
  --content "Progress update or decision note" \
  --pretty

# Start time tracking
.pm/bin/pm time-entry create \
  --work-item-id <work-item-id> \
  --description "What you're working on" \
  --pretty

# Stop time tracking
.pm/bin/pm time-entry update <time-entry-id> --stop --pretty
```

### Work Item Types

- **Epic**: Large features or initiatives (e.g., "Implement transcription pipeline")
- **Story**: User-facing features or capabilities (e.g., "Add support for MP3 files")
- **Task**: Implementation work, bugs, or technical tasks (e.g., "Fix audio buffer overflow")

### Best Practices

1. **Organize work hierarchically**: Create epics for major features, then break them down into stories and tasks
2. **Use priorities**: Mark urgent work as `high` or `critical` priority
3. **Capture context**: Add comments to work items to preserve decisions and discussions
4. **Track dependencies**: Use dependency links when work items block each other
5. **Plan sprints**: Organize work into time-boxed sprints with clear goals

### Translating Implementation Plans into Agile Work Items

When you have a detailed implementation plan (like a plan file from plan mode), organize it into /pm using this structure:

#### Epic → Story → Task Hierarchy

**Epics** (High-level components):
- Create epics for major system components or feature areas
- Examples: "Core STT Library", "Binary Application", "Web API & Blazor Frontend"
- Use `--type epic` when creating

**Stories** (Logical implementation groupings):
- Group related tasks together within an epic using `--parent-id <epic-id>`
- Each story should include:
  - **Brief summary**: What this story accomplishes and WHY it's needed
  - **Implementation order note**: Dependencies on other stories, what must be done first
  - Example: "Foundation - Error Handling & Crate Structure. This MUST be done first as all components depend on proper error handling."
- Use `--type story` when creating

**Tasks** (Specific implementation work):
- Create concrete work items under stories using `--parent-id <story-id>`
- **CRITICAL**: Include actual production-grade code snippets in the description
- Each task description should contain:
  - What needs to be implemented
  - **Actual Rust code in markdown code blocks** (copy-pasteable, production-grade)
  - File path where this code goes
  - Key patterns to follow (error handling, tracing, etc.)
- Use `--type task` when creating

**Example Task Description Format**:
```markdown
Implement AudioError enum with location tracking for debugging.

File: `crates/auto-scribe-core/src/error.rs`

```rust
use error_location::ErrorLocation;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("No microphone found {location}")]
    NoMicrophoneFound {
        location: ErrorLocation,
    },
    // ... additional variants
}

pub type Result<T> = std::result::Result<T, AudioError>;
```

Key patterns:
- Every error variant MUST have `location: ErrorLocation` field
- Use `#[track_caller]` on all sync functions returning Result
- Use `ErrorLocation::from(Location::caller())` when creating errors
```

#### Implementation Order & Dependencies

1. **Use story descriptions** to indicate sequencing: "This must be done after Story X" or "Can be done in parallel with Story Y"
2. **Create dependency links** between work items using the dependency commands when there are hard blockers
3. **Follow dependency layers** from the plan: Foundation → Core Components → Integration → Application
4. **Organize by critical path**: Identify the shortest path to a working feature and prioritize accordingly

#### Benefits of This Approach

- **Future-proof**: Code snippets preserve implementation details across sessions
- **Clear context**: Stories explain WHY work needs to be done and WHEN
- **Production-grade**: Code examples follow best practices (/pro-rust patterns)
- **Proper organization**: Hierarchical structure matches agile methodology
- **Reduces mistakes**: Concrete code examples prevent misunderstandings

For full `/pm` skill documentation, use the skill directly.

## Rust Development

**IMPORTANT:** Use the `/pro-rust` skill when writing Rust code to ensure production-grade patterns and best practices. This skill provides guidance on proper Rust architecture, error handling, and idiomatic code patterns.

## Development Commands

### Building
```bash
cargo build              # Build all workspace members
cargo build --release    # Build optimized release version
cargo build -p <crate>   # Build specific crate
```

### Testing
```bash
cargo test                    # Run all tests
cargo test -p <crate>        # Run tests for specific crate
cargo test <test_name>       # Run specific test
cargo test -- --nocapture    # Show println! output
```

### Running
```bash
cargo run -p <crate>         # Run specific binary crate
```

### Code Quality
```bash
cargo clippy                 # Run linter
cargo clippy --all-targets   # Lint all targets including tests
cargo fmt                    # Format code
cargo fmt -- --check         # Check formatting without modifying
```

### Other Useful Commands
```bash
cargo check                  # Fast compilation check without producing binary
cargo clean                  # Remove build artifacts
cargo tree                   # Show dependency tree
```

## Workspace Structure

The `Cargo.toml` at the root defines a workspace with:
- Edition: 2024
- License: MIT
- Repository: https://github.com/TonyMarkham/auto-scribe

When adding new crates, update the `members` array in the root `Cargo.toml`.

## Adding New Crates

To add a new crate to the workspace:
```bash
cargo new <crate-name>           # For binary
cargo new --lib <crate-name>     # For library
```

Then add the crate path to `members = []` in the root Cargo.toml.

## Workspace Dependencies

Shared dependencies should be defined in `[workspace.dependencies]` and referenced in individual crates using `workspace = true`.
