# REQUIREMENTS.md

## Functional Requirements

### Diagram Types

- **State Machine Diagram** - Design hierarchical state machines
- **Sequence Diagram** - Model object interactions over time
- **Use Case Diagram** - Capture system functionality and actors
- **Activity Diagram** - Model workflows and business processes

### Diagram Editing

- Create diagrams of any supported type
- Support multiple diagrams via tabbed interface with type icons
- Add, move, resize, and delete diagram elements
- Align and organize shapes (left, right, center, top, bottom, middle)
- Select single or multiple elements
- Marquee selection (requires full containment)
- Lasso selection tool for freeform selection
- Ctrl+Click to toggle selection / add to selection
- Ctrl+Arrow keys to nudge selected elements by 1 pixel
- Undo/redo operations
- Dynamic toolbar showing diagram-type-specific tools

### State Machine Elements

#### States
- Simple states with name
- Entry, exit, and do activities
- Composite states containing regions with substates
- Visual representation as rounded rectangles
- Customizable fill colors

#### Pseudo-states
- Initial state (filled circle - starting point)
- Final state (double circle - terminal point)
- Choice (diamond - conditional branching)
- Fork (bar - parallel split)
- Join (bar - parallel merge)
- Junction (diamond - unconditional branching)

#### Regions
- States can contain regions for parallel substates
- Region separator lines are draggable to resize regions
- Nodes placed in a state become children of that state's region
- Parent-child relationships automatically maintained on drag
- Size-based z-order (smaller states render on top of larger ones)

#### Transitions
- Connect states with directional arrows
- Event trigger
- Guard condition
- Action specification
- Automatic routing with stubs
- Draggable connection labels with optional leader lines

### Sequence Diagram Elements

#### Lifelines
- Represent objects/participants in interactions
- Visual: Rectangle head with dashed vertical line
- Optional stereotype and destruction marker

#### Messages
- Synchronous messages (filled arrowhead)
- Asynchronous messages (open arrowhead)
- Return messages (dashed line)
- Self messages (loop back to same lifeline)

#### Activations
- Execution occurrence boxes on lifelines
- Can be nested

#### Combined Fragments
- alt, opt, loop, par, break, critical, neg, assert, ignore, consider

### Use Case Diagram Elements

#### Actors
- Stick figure representation
- Named entities external to system

#### Use Cases
- Ellipse representation
- Named system functions

#### System Boundary
- Rectangle containing use cases
- Named system context

#### Relationships
- Association (solid line)
- Include (dashed arrow with <<include>>)
- Extend (dashed arrow with <<extend>>)
- Generalization (solid line with hollow triangle)

### Activity Diagram Elements

#### Actions
- Rounded rectangle actions
- Call behavior actions
- Send/accept signal actions (pentagon shapes)
- Accept time event actions

#### Control Nodes
- Initial node (filled circle)
- Final node (double circle)
- Decision/merge nodes (diamond)
- Fork/join bars

#### Swimlanes
- Vertical or horizontal partitions
- Named responsibility areas

#### Object Nodes
- Central buffer nodes
- Data store nodes
- Input/output pins

### File Operations
- Save diagrams to JSON format (.jmt extension)
- Load diagrams from JSON files
- Save and Save As with file dialog
- Auto-save support (planned)

### Export
- Export diagrams as PNG images
- Optional autocrop to remove excess whitespace
- Convert menu for export options

## Non-Functional Requirements

### Platform
- Desktop: Linux, macOS, Windows
- Web: Browser via WASM (planned)

### Architecture
- Client-server split for WASM support
- Protobuf communication between client and server
- Desktop spawns local server; WASM connects to remote server

### User Interface
- Graphical editor with canvas
- Properties/attributes panel
- Menu bar and toolbar
- Status bar showing current mode

### Performance
- Responsive diagram editing
- Efficient rendering via immediate mode GUI
- Fast serialization/deserialization

## Technical Requirements

### Rust Implementation
- Cargo workspace with 5 crates:
  - `jmt-core`: Shared data types (geometry, nodes, connections, diagrams)
  - `jmt-proto`: Protobuf definitions for client-server communication
  - `jmt-server`: WebSocket server for file operations
  - `jmt-client`: egui frontend (WASM-compatible)
  - `jmt-desktop`: Desktop launcher

### Dependencies
- `eframe`/`egui` for GUI
- `serde`/`serde_json` for serialization
- `prost` for protobuf
- `tokio` for async runtime
- `tokio-tungstenite` for WebSocket
