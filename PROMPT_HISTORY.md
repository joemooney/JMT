# PROMPT_HISTORY.md

## Session 1 - Initial Setup (2025-12-19)

### Prompt: Initialize project documentation

**Actions Taken:**
1. Explored codebase structure to understand the project
2. Created CLAUDE.md with project guidance for Claude Code
3. Created OVERVIEW.md with project vision and general information
4. Created REQUIREMENTS.md with functional and non-functional requirements
5. Created PROMPT_HISTORY.md to track development sessions

**Files Created:**
- CLAUDE.md
- OVERVIEW.md
- REQUIREMENTS.md
- PROMPT_HISTORY.md

---

## Session 2 - Rust Conversion (2025-12-19)

### Prompt: Convert to Rust egui application

**User Request:**
Convert the Fantom state machine diagram editor to a Rust application using egui, with:
- JSON file format for saving diagrams
- Skip code generation for now (focus on editor)
- Client-server split with protobuf for WASM support
- Desktop spawns local server; WASM connects to remote server
- Fresh start (no legacy file import needed)

**Actions Taken:**
1. Explored existing Fantom codebase thoroughly:
   - 27 Fantom files, ~15,000+ lines
   - Analyzed node hierarchy, connection system, rendering patterns
   - Documented graphics API mappings for egui conversion

2. Created implementation plan with 10 phases

3. Implemented Phase 1: Foundation
   - Created Cargo workspace with 5 crates
   - Implemented `jmt-core`: geometry, nodes, connections, diagrams
   - Implemented `jmt-proto`: protobuf definitions for client-server
   - Implemented `jmt-server`: WebSocket server for file operations
   - Implemented `jmt-client`: egui frontend with full UI
   - Implemented `jmt-desktop`: desktop launcher

**Rust Project Structure Created:**
```
jmt-rust/
├── Cargo.toml              # Workspace configuration
├── jmt-core/               # Core data types
│   └── src/
│       ├── lib.rs
│       ├── geometry.rs     # Point, Rect, Color
│       ├── edit_mode.rs    # EditMode enum
│       ├── settings.rs     # DiagramSettings
│       ├── connection.rs   # Connection, LineSegment
│       ├── diagram.rs      # Diagram container
│       └── node/
│           ├── mod.rs      # Node enum, NodeType, Side, Corner
│           ├── state.rs    # State struct
│           ├── region.rs   # Region struct
│           └── pseudo.rs   # PseudoState, PseudoStateKind
├── jmt-proto/              # Protobuf definitions
│   ├── proto/
│   │   ├── diagram.proto
│   │   └── commands.proto
│   └── src/
│       ├── lib.rs
│       └── jmt.rs          # Generated protobuf code
├── jmt-server/             # Backend server
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── file_ops.rs
│       └── service.rs
├── jmt-client/             # egui frontend
│   └── src/
│       ├── lib.rs
│       ├── app.rs          # Main application
│       ├── server_client.rs
│       ├── canvas/
│       │   ├── mod.rs
│       │   └── renderer.rs
│       └── panels/
│           ├── mod.rs
│           ├── menu_bar.rs
│           ├── toolbar.rs
│           ├── properties.rs
│           └── status_bar.rs
└── jmt-desktop/            # Desktop launcher
    └── src/main.rs
```

**Features Implemented:**
- Complete node hierarchy (State, Initial, Final, Choice, Fork, Join, Junction)
- Regions for composite states
- Connections with automatic routing
- Canvas rendering with egui Painter
- Node selection, movement, and deletion
- Undo/redo with serialization-based snapshots
- Toolbar with all edit modes
- Properties panel for editing node/connection/diagram properties
- Tab-based multi-diagram support
- Alignment tools (left, right, center, top, bottom, middle)
- Keyboard shortcuts (Delete, Ctrl+Z, Ctrl+Shift+Z, Escape)

**Build Status:**
- Successfully compiles with `cargo build --release`
- All 5 crates building correctly

**Git Operations:**
- Committed: `b15eea8` - Add Rust/egui implementation of JMT state machine editor
- Pushed to GitHub: master branch

---

## Session 3 - Marquee Selection (2025-12-19)

### Prompt: Add rectangular marquee selection

**User Request:**
Add the ability to drag the mouse and select all elements in a rectangular area.

**Actions Taken:**
1. Added `SelectionRect` struct in `jmt-client/src/app.rs` to track selection rectangle state
2. Added `select_nodes` and `select_nodes_in_rect` methods to `Diagram` in `jmt-core/src/diagram.rs`
3. Modified drag handling in `JmtApp` to detect:
   - Drag on node → move selected nodes
   - Drag on empty canvas → marquee selection
4. Added visual rendering of selection rectangle (blue semi-transparent)
5. On drag end, selects all nodes within the rectangle

**Files Modified:**
- `jmt-core/src/diagram.rs` - Added multi-select methods
- `jmt-client/src/app.rs` - Added SelectionRect and marquee drag handling

**Git Operations:**
- Committed and pushed marquee selection feature

---

## Session 3 (continued) - Undo/Redo Buttons (2025-12-19)

### Prompt: Add undo and redo buttons

**User Request:**
Add undo and redo buttons for undoing/redoing the last action.

**Actions Taken:**
1. Added Undo and Redo buttons to the toolbar (at the beginning)
2. Buttons are disabled when undo/redo is not available
3. Buttons show tooltip with keyboard shortcuts (Ctrl+Z, Ctrl+Shift+Z)

**Files Modified:**
- `jmt-client/src/panels/toolbar.rs` - Added Undo/Redo buttons

**Git Operations:**
- Committed and pushed undo/redo buttons

---

## Session 3 (continued) - Click-and-Drag Fix (2025-12-19)

### Prompt: Fix click-and-drag behavior

**User Request:**
Clicking and dragging should immediately drag the element, not require click-release-drag.

**Actions Taken:**
1. Modified drag_started handler to select the node under cursor if not already selected
2. Push undo at drag start (before movement) instead of drag end
3. Now click-and-drag works in a single motion

**Files Modified:**
- `jmt-client/src/app.rs` - Fixed drag handling to select on drag start

**Git Operations:**
- Committed and pushed click-and-drag fix

---

## Session 3 (continued) - Drag Switches to Arrow Mode (2025-12-19)

### Prompt: Dragging node should exit Add mode

**User Request:**
If in Add: State mode and start dragging a node, should exit Add mode and switch to Arrow/select mode.

**Actions Taken:**
1. Modified drag_started handler to check for node under cursor regardless of edit mode
2. If dragging on a node in any mode, switch to Arrow mode first, then start dragging
3. Marquee selection only starts in Arrow mode (not when in Add modes)

**Files Modified:**
- `jmt-client/src/app.rs` - Switch to Arrow mode when dragging existing node

**Git Operations:**
- Committed and pushed mode switch on drag

---

## Session 3 (continued) - Align Dropdown and Distribute (2025-12-19)

### Prompt: Convert Align buttons to dropdown with icons and add distribute

**User Request:**
Make the Align: buttons a dropdown under align and add an icon for each. Also add a distribute option to evenly space the nodes.

**Actions Taken:**
1. Converted Align buttons to a dropdown menu
2. Added Unicode icons to all toolbar buttons:
   - ⟲ Undo, ⟳ Redo
   - ↖ Arrow, ▢ State, ● Initial, ◉ Final, ◇ Choice, ◆ Junction, ┳ Fork, ┻ Join
   - → Transition
3. Align dropdown contains:
   - Horizontal: ⫷ Left, ⫿ Center, ⫸ Right
   - Vertical: ⫠ Top, ⫟ Middle, ⫡ Bottom
   - Distribute: ↔ Horizontally, ↕ Vertically
4. Align dropdown is disabled when fewer than 2 nodes selected
5. Distribute requires at least 3 nodes to work

**Files Modified:**
- `jmt-client/src/panels/toolbar.rs` - Complete toolbar redesign with icons and dropdown

**Git Operations:**
- Committed and pushed align dropdown and distribute

---

## Session 3 (continued) - Auto-size States and Activity Visibility (2025-12-19)

### Prompt: State size should grow to accommodate text, add show/hide activities

**User Request:**
The size of the state should grow to accommodate the text. Have a checkbox per state (in properties) and per diagram (View menu) to show or hide the entry/exit/do activities.

**Actions Taken:**
1. Added `show_activities` field to State (Option<bool> - None uses diagram default)
2. Added `show_activities` field to DiagramSettings (global default, true by default)
3. Added `calculate_required_size()` and `resize_to_fit()` methods to State
4. Updated renderer to check visibility settings and support multi-line activities
5. Added to Properties panel:
   - "Show Activities" checkbox per state
   - "Use diagram default" button to reset
   - "Fit to Content" button to auto-resize state
6. Added to View menu:
   - "Show Activities" checkbox for diagram-wide default

**Files Modified:**
- `jmt-core/src/node/state.rs` - Added show_activities, size calculation methods
- `jmt-core/src/settings.rs` - Added show_activities to DiagramSettings
- `jmt-client/src/canvas/renderer.rs` - Check visibility, multi-line support
- `jmt-client/src/panels/properties.rs` - Added checkbox and fit button
- `jmt-client/src/panels/menu_bar.rs` - Added View > Show Activities

**Git Operations:**
- Committed and pushed auto-size and activity visibility

---

## Session 3 (continued) - Cursor Preview for Add Modes (2025-12-19)

### Prompt: Show ghost preview of element being added at cursor

**User Request:**
When adding a state or pseudo-state, show a dummy preview at the cursor position. ESC returns to Arrow mode.

**Actions Taken:**
1. Added `cursor_pos` field to JmtApp to track mouse position on canvas
2. Added `render_cursor_preview()` method to draw ghost elements:
   - State: Semi-transparent rounded rectangle with "State" label
   - Initial: Semi-transparent filled circle
   - Final: Semi-transparent double circle
   - Choice/Junction: Semi-transparent diamond
   - Fork/Join: Semi-transparent horizontal bar
   - Connect: Crosshair or orange circle (when waiting for target)
3. Preview follows cursor in real-time
4. ESC already returns to Arrow mode (was already implemented)

**Files Modified:**
- `jmt-client/src/app.rs` - Added cursor tracking and preview rendering

**Git Operations:**
- Committed and pushed cursor preview

**Next Steps (Future Sessions):**
- Phase 3: Node resize from corners
- Phase 4: Improve pseudo-state rendering
- Phase 5: Connection label positioning
- Phase 6: Region separator dragging
- Phase 7: Polish undo/redo
- Phase 8: Server integration with actual WebSocket communication
- Phase 9: WASM build configuration
- Phase 10: Desktop integration refinements
