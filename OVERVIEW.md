# OVERVIEW.md

## Vision

JMT (Joe's Modeling Toolkit) is a visual UML modeling toolkit that enables developers to design various UML diagram types. The project is being rewritten in Rust with egui for a modern, cross-platform experience including web browser support.

## What This Project Does

- Provides a graphical user interface for creating UML diagrams
- Supports multiple diagram types:
  - **State Machine Diagrams** - states, transitions, pseudo-states (initial, final, choice, fork, join, junction)
  - **Sequence Diagrams** - lifelines, messages (sync/async/return), activations, combined fragments
  - **Use Case Diagrams** - actors, use cases, system boundaries, relationships (association, include, extend, generalization)
  - **Activity Diagrams** - actions, decisions, forks/joins, swimlanes, object nodes, control flows
- Enables hierarchical state machines with regions and nested substates
- Supports multiple diagrams via tabbed interface with diagram type indicators

## Target Users

Developers who need to design and implement state machines in their projects.

## Technology Stack

### Current: Rust + egui (in development)
- **Language**: Rust
- **GUI**: egui (immediate mode GUI framework)
- **Serialization**: JSON via serde
- **Architecture**: Client-server with protobuf for WASM support
- **Platforms**: Desktop (Linux, macOS, Windows) and Web Browser (WASM)

### Legacy: Fantom + FWT
The original implementation uses Fantom (JVM) with FWT for the GUI. See `fan/` directory.

## Key Features

1. **Multiple Diagram Types** - State machine, sequence, use case, and activity diagrams
2. **Visual Editing** - Click to add, drag to move, resize from corners
3. **Dynamic Toolbars** - Context-aware tools based on diagram type
4. **State Types** - Simple states, composite states with regions
5. **Transitions** - Connections with events, guards, and actions
6. **Pseudo-states** - Initial, final, choice, fork, join, junction nodes
7. **Multi-diagram** - Tab-based interface with diagram type icons
8. **Undo/Redo** - Full undo/redo support
9. **Selection Tools** - Marquee and lasso selection, Ctrl+Click for multi-select
10. **Zoom Controls** - Toolbar buttons and Ctrl+MouseWheel for zoom (25%-400%)
11. **Canvas Scrolling** - Scrollbars when content exceeds view
12. **Web Support** - WASM build for browser-based editing (planned)

## Project Structure

```
JMT/
├── fan/                    # Legacy Fantom source code
│   ├── images/             # UI icons
│   └── *.fan               # 26 Fantom source files
├── jmt-rust/               # New Rust implementation
│   ├── jmt-core/           # Shared data types and domain logic
│   ├── jmt-proto/          # Protobuf definitions
│   ├── jmt-server/         # Backend server for file operations
│   ├── jmt-client/         # egui frontend
│   └── jmt-desktop/        # Desktop launcher
└── docs/                   # Documentation
```

## Getting Started

### Rust Version (Recommended)
```bash
cd jmt-rust
cargo run --release
```

### Legacy Fantom Version
```bash
fan build.fan
fan JsmGui
```
