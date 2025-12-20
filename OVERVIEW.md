# OVERVIEW.md

## Vision

JMT (Joe's Modeling Toolkit) is a visual state machine modeling toolkit that enables developers to design UML-style hierarchical state machine diagrams. The project is being rewritten in Rust with egui for a modern, cross-platform experience including web browser support.

## What This Project Does

- Provides a graphical user interface for creating state machine diagrams
- Supports UML state machine concepts: states, transitions, pseudo-states (initial, final, choice, fork, join, junction)
- Enables hierarchical state machines with regions and nested substates
- Supports multiple diagrams via tabbed interface

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

1. **Visual Editing** - Click to add, drag to move, resize from corners
2. **State Types** - Simple states, composite states with regions
3. **Transitions** - Connections with events, guards, and actions
4. **Pseudo-states** - Initial, final, choice, fork, join, junction nodes
5. **Multi-diagram** - Tab-based interface for multiple diagrams
6. **Undo/Redo** - Full undo/redo support
7. **Web Support** - WASM build for browser-based editing (planned)

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
