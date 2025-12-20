# REQUIREMENTS.md

## Functional Requirements

### Diagram Editing

- Create and edit state machine diagrams
- Support multiple diagrams via tabbed interface
- Add, move, resize, and delete diagram elements
- Align and organize shapes
- Select single or multiple elements
- Undo/redo operations

### State Machine Elements

#### States
- Simple states with name
- Entry, exit, and do activities
- Composite states containing regions with substates
- Visual representation as rounded rectangles

#### Pseudo-states
- Initial state (starting point)
- Final state (terminal point)
- Choice (conditional branching)
- Fork (parallel split)
- Join (parallel merge)
- Junction (unconditional branching)

#### Transitions
- Connect states with directional arrows
- Event trigger
- Guard condition
- Action specification
- Multiple rendering styles (LINE, ARC, U_SHAPE, L_SHAPE, STEP, S_SHAPE)

### Code Generation
- Generate C++ code from state machine diagram
- Output event enums
- Output state variables
- Output transition definitions
- Output initial transitions

### File Operations
- Save diagrams to file
- Load diagrams from file
- Backup functionality

## Non-Functional Requirements

### Platform
- Cross-platform via JVM
- Fantom runtime required

### User Interface
- Graphical editor with canvas
- Properties/attributes panel
- Menu bar and toolbar
- Status bar

### Performance
- Responsive diagram editing
- Efficient rendering of complex hierarchical states
