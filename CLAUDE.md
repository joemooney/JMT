# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

JMT (Joe's Modeling Toolkit) is a state machine diagram editor. The project has two implementations:
1. **Rust + egui** (in `jmt-rust/`) - Active development, modern cross-platform including WASM
2. **Fantom + FWT** (in `fan/`) - Legacy implementation

## Build & Run Commands

### Rust Version (Recommended)
```bash
cd jmt-rust
cargo build --release
cargo run --release
# or
./target/release/jmt
```

### Legacy Fantom Version
```bash
fan build.fan
fan JsmGui
```

## Tech Stack - Rust Implementation

- **Language**: Rust 2021 edition
- **GUI**: egui/eframe (immediate mode GUI)
- **Serialization**: serde + serde_json
- **Server Communication**: WebSocket + protobuf (prost)
- **Async Runtime**: tokio

## Architecture - Rust

### Cargo Workspace (5 crates)

1. **jmt-core** - Shared data types and domain logic
   - `geometry.rs` - Point, Rect, Color
   - `node/` - Node, State, Region, PseudoState
   - `connection.rs` - Connection, LineSegment
   - `diagram.rs` - Diagram container
   - `edit_mode.rs` - EditMode enum
   - `settings.rs` - DiagramSettings

2. **jmt-proto** - Protobuf definitions for client-server communication
   - `proto/diagram.proto` - Diagram structures
   - `proto/commands.proto` - Client-server commands

3. **jmt-server** - Backend server for file operations
   - WebSocket server
   - File I/O (save/load JSON)

4. **jmt-client** - egui frontend (WASM-compatible)
   - `app.rs` - Main application state
   - `canvas/renderer.rs` - Diagram rendering
   - `panels/` - Menu bar, toolbar, properties, status bar

5. **jmt-desktop** - Desktop launcher
   - Spawns server and runs client

### Node Hierarchy

- `Node` enum containing `State` or `Pseudo`
- `State` - Rounded rectangles with name, activities, regions
- `PseudoState` - Initial (circle), Final (double circle), Choice/Junction (diamond), Fork/Join (bar)
- `Region` - Container for child nodes within composite states
- `Connection` - Transitions with event, guard, action

### Key Patterns

- Serde for JSON serialization
- UUID for node/connection IDs
- EditMode enum for current editing state
- Undo/redo via diagram serialization snapshots

## Key Files

| Purpose | File |
|---------|------|
| Main app | `jmt-client/src/app.rs` |
| Rendering | `jmt-client/src/canvas/renderer.rs` |
| Core types | `jmt-core/src/` |
| Server | `jmt-server/src/service.rs` |
| Desktop entry | `jmt-desktop/src/main.rs` |

## Development Notes

- Client-server split enables future WASM support
- All editing happens in client; server only handles file I/O
- Properties panel auto-switches based on selection
- Connections auto-route when nodes move
