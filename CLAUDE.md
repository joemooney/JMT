# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

JMT (Joe's Modeling Toolkit) is a state machine diagram editor written in Fantom. It provides a GUI for creating UML-style state machine diagrams and can generate C++ code from them.

## Build & Run Commands

```bash
# Build the pod
fan build.fan

# Run the application
fan JsmGui
```

## Tech Stack

- **Language**: Fantom (JVM-based)
- **UI**: FWT (Fantom Widget Toolkit)
- **Graphics**: GFX module
- **Pod Name**: JsmGui
- **Dependencies**: sys 1.0, gfx 1.0, fwt 1.0

## Architecture

### Core Components

1. **JsmGui** (`fan/JsmGui.fan`) - Main application entry point, manages tabbed interface with multiple diagrams
2. **JsmDiagram** (`fan/JsmDiagram.fan`) - Represents individual state machine diagram with settings and canvas
3. **JsmCanvas** (`fan/JsmCanvas.fan`) - Base canvas class handling mouse/keyboard events
4. **StateMachineCanvas** (`fan/StateMachineCanvas.fan`) - Main canvas for drawing state machines

### Node Hierarchy

- **JsmNode** - Base class for all diagram elements (coordinates, collision detection, drawing)
- **JsmSmNode** - State machine-specific node behavior
- **JsmState** - State nodes with regions, entry/exit/do activities, supports hierarchical substates
- **JsmRegion** - Container for child states within composite states
- **Pseudo-states**: JsmInitial, JsmFinal, JsmChoice, JsmFork, JsmJoin, JsmJunction

### Connections

- **JsmConnection** - Transitions between states (event, guard, action properties)
- **JsmLineSegment** - Line segments making up connections
- Rendering styles: LINE, ARC, U_SHAPE, L_SHAPE, STEP, S_SHAPE

### Code Generation

- **JsmGenerator** (`fan/JsmGenerator.fan`) - Generates C++ code including event enums, state variables, transition definitions

## Key Patterns

- All classes use `Jsm` prefix
- Serializable classes marked with `@Serializable`, transient fields with `@Transient`
- Two constructor patterns: `make(|This| f)` and `maker(...)`
- 14 edit modes defined in EditMode enum (ARROW, SELECT, ADD_STATE, CONNECT, etc.)

## Technical Limitations

- Backup/project paths have Windows-style hardcoded paths (`c:/jsm/`)
