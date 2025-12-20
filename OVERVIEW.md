# OVERVIEW.md

## Vision

JMT (Joe's Modeling Toolkit) is a visual state machine modeling toolkit that enables developers to design UML-style hierarchical state machine diagrams and generate C++ code from them.

## What This Project Does

- Provides a graphical user interface for creating state machine diagrams
- Supports UML state machine concepts: states, transitions, pseudo-states (initial, final, choice, fork, join, junction)
- Enables hierarchical state machines with regions and nested substates
- Generates C++ code from designed state machines
- Supports multiple diagrams via tabbed interface

## Target Users

Developers who need to design and implement state machines, particularly in C++ projects.

## Technology Choice

The project uses Fantom, a JVM-based language, with FWT (Fantom Widget Toolkit) for the GUI. This provides cross-platform capability through the JVM.

## Key Features

1. **Visual Editing** - Drag/drop, resize, alignment tools for diagram elements
2. **State Types** - Simple states, composite states with regions
3. **Transitions** - Connections with events, guards, and actions
4. **Pseudo-states** - Initial, final, choice, fork, join, junction nodes
5. **Code Generation** - Export to C++ code
6. **Multi-diagram** - Tab-based interface for multiple diagrams

## Project Structure

```
JMT/
├── build.fan          # Fantom pod build configuration
├── fan/               # Source code directory
│   ├── images/        # UI icons (16 PNG files)
│   └── *.fan          # 26 Fantom source files (~7,931 lines)
```

## Getting Started

1. Ensure Fantom is installed
2. Build: `fan build.fan`
3. Run: `fan JsmGui`
