using gfx
using fwt

enum class ActiveType { STATE, DIAGRAM, CONN }

  
class JsmDiagram
{
  ActiveType activeType
  JsmAttributes attributes
  StateMachineCanvas stateMachineCanvas
  BorderPane stateMachinePane
  BorderPane attributesPane
  BorderPane diagramCanvas
  SashPane mainPane
  Tab? diagramTab
  Str? redrawReason
  JsmDiagramSettings settings
  EditMode? mode
  Button? currentButton // used to deselect button when changing to another palette button
  JsmGui gui

  new make(JsmGui gui, Str newDiagramName, Str newDiagramPath)
  {
    this.gui=gui
    this.settings=JsmDiagramSettings()
    this.settings.diagramName = newDiagramName
    this.settings.diagramPath = newDiagramPath
    echo("diagramPath $newDiagramPath")
    if ( newDiagramPath == "" )
    {
      //this.settings.diagramPath=JsmOptions.instance.projectPath+"/"+newDiagramPath+".txt"
    }
    
    attributes = JsmAttributes(this)
    activeType=ActiveType.STATE
    
    stateMachineCanvas =  StateMachineCanvas(gui,this) { name = "A"; demo = gui }
    
    stateMachinePane =  BorderPane {
       stateMachineCanvas ,
    }
    //attributes.rootState=stateMachineCanvas.rootState
    
    stateMachinePane.bg=Color.purple
    
    attributesPane = BorderPane
    {
      //border = Border.defVal
      border = Border("#000")
      insets = Insets(5)
      content = attributes.statePane
    }

    diagramCanvas = BorderPane
    {
      border = Border("#000")
      insets = Insets(0)
      content = stateMachinePane
    }

    //
    mainPane= SashPane
    {
      weights = [7,4]
      diagramCanvas,
      attributesPane,
    }
    //this.incSave(); // save initial state to roll back to

  }
  
  ** Are there unsaved changes to the diagram
  Bool notSaved()
  {
    return(this.attributes.notSaved)
  }
  
  JsmState getRootState()
  {
    return(this.stateMachineCanvas.rootState)
  }
  
  Void performAlign(AlignMode alignMode)
  {
    Bool moved:=stateMachineCanvas.performAlign(alignMode); 
    if ( moved ) 
    {
      this.redrawReason="align"
      this.incSave()
    }
  }
  
  Void checkRedraw()
  {
    if ( this.redrawReason != null )
    {
      stateMachineCanvas.redraw(this.redrawReason)
      this.redrawReason=null
    }
  }
  
    
  Void setEditMode(EditMode mode)
  {
    this.stateMachineCanvas.deselectNodes()
    this.stateMachineCanvas.setCurrentNode(null)
    this.redrawReason="clicked new mode"
    setMode(mode)
  }
  
    // change the edit mode 
  Void setMode(EditMode mode)
  {
    if ( this.currentButton != null )
    {
      this.currentButton.selected=false
    }
    echo("Setting mode $mode") 
    switch(mode)
    {
      case EditMode.ADD_STATE:
        this.stateMachineCanvas.cursor=Cursor(gui.stateIcon,8,8)
        this.currentButton=gui.stateButton
      case EditMode.ADD_JOIN:
        this.stateMachineCanvas.cursor=Cursor(gui.joinIcon,8,8)
        this.currentButton=gui.joinButton
      case EditMode.ADD_CHOICE:
        this.stateMachineCanvas.cursor=Cursor(gui.choiceIcon,8,8)
        this.currentButton=gui.choiceButton
      case EditMode.ADD_JUNCTION:
        this.stateMachineCanvas.cursor=Cursor(gui.junctionIcon,8,8)
        this.currentButton=gui.junctionButton
      case EditMode.ARROW:
        this.stateMachineCanvas.cursor=Cursor.defVal
        this.currentButton=gui.cursorButton
      case EditMode.CONNECT:
        this.stateMachineCanvas.cursor=Cursor(gui.transitionIcon,8,8)
        this.currentButton=gui.transitionButton
      case EditMode.ENTER_CONNECT:
        this.stateMachineCanvas.cursor=Cursor(gui.transitionIcon,8,8)
        this.currentButton=gui.transitionButton
      case EditMode.ADD_FORK:
        this.stateMachineCanvas.cursor=Cursor(gui.forkIcon,8,8)
        this.currentButton=gui.forkButton
      case EditMode.ADD_INITIAL:
        this.stateMachineCanvas.cursor=Cursor(gui.initialIcon,8,8)
        this.currentButton=gui.initialButton
      case EditMode.ADD_FINAL:
        this.stateMachineCanvas.cursor=Cursor(gui.finalIcon,8,8)
        this.currentButton=gui.finalButton
      case EditMode.RESIZE:
        this.stateMachineCanvas.cursor=Cursor.seResize
        this.currentButton=gui.cursorButton
      default:
        echo("no change to cursor for mode $mode")
    }
    this.currentButton.selected=true
    this.stateMachineCanvas.mode=mode
    this.mode=mode
  }
  
  ** Restore a diagram from disk
  Void restoreState(JsmState s)
  {
    this.settings=s.settings
    this.stateMachineCanvas.rootNode=s
    this.stateMachineCanvas.rootState=s
    if ( s.settings == null )
    {
      echo("[error] Restored a non-root state")
    }
    this.stateMachineCanvas.restore(s) 
    this.gui.redoButton.enabled=false;
    this.gui.undoButton.enabled=false;
    this.stateMachineCanvas.repaint
  }

  Void undoAction()
  {
    JsmState? newRootState:=this.attributes.incUndo();
    if ( newRootState != null )
    {
      this.stateMachineCanvas.restore(newRootState)
      this.stateMachineCanvas.repaint()
      if ( this.attributes.lastInc.size > 0 )
      {
        this.gui.undoButton.enabled=true;
      }
      else
      {
        this.gui.undoButton.enabled=false;
      }
      this.gui.redoButton.enabled=true;
    }
  }
  
  Void redoAction()
  {
    JsmState? newRootState:=this.attributes.incRedo();
    if ( newRootState != null )
    {
      this.stateMachineCanvas.restore(newRootState)
      this.stateMachineCanvas.repaint()
      if ( this.attributes.redoInc.size > 0 )
      {
        this.gui.redoButton.enabled=true;
      }
      else
      {
        this.gui.redoButton.enabled=false;
      }
      this.gui.undoButton.enabled=false;
    }
  }
  
  virtual Void updateAttributes()
  {
    JsmNode? activeNode:=this.stateMachineCanvas.currentNode
    JsmConnection? activeConn
    if ( activeNode == null )
    {
      activeNode=this.stateMachineCanvas.rootNode
    }
    echo("Active node is $activeNode.name , $stateMachineCanvas.selectedConns.size")
    
    // no current node selected and 
    // display diagram attributes if 
    // root node && selected conns != 1
    // !root node  && selected conns == 1
    if (
         ( activeNode == this.stateMachineCanvas.rootNode && this.stateMachineCanvas.selectedConns.size != 1 )
        ||
         ( activeNode != this.stateMachineCanvas.rootNode && this.stateMachineCanvas.selectedConns.size != 0 ) 
      )
    {
      echo("Displaying diagram attributes - root selected")
      showDiagramAttributes() 
    }
    else if( this.stateMachineCanvas.selectedConns.size == 1 )
    {
      echo("Displaying conn attributes")
      activeConn=this.stateMachineCanvas.selectedConns.first()
      showConnAttributes() 
    }
    else if ( activeNode !=null && ( activeNode.type == NodeType.STATE || isPseudoState(activeNode)) )
    {
      echo("Displaying state attributes")
      showStateAttributes() 
    }
    else
    {
      echo("Displaying Diagram attributes")
      showDiagramAttributes() 
    }
  }
  
  Bool isPseudoState(JsmNode n)
  {
    if ( n.typeof.fits(JsmPseudoState#) )
    {
      return(true)
    }
    else
    {
      return(false)
    }
  }
  Void showDiagramAttributes()
  {
    JsmState? rootState:=this.stateMachineCanvas.rootState
    JsmConnection? activeConn := this.stateMachineCanvas.selectedConns.first()
    
    if ( activeType!=ActiveType.DIAGRAM)
    {
        activeType=ActiveType.DIAGRAM
        this.attributesPane.content = this.attributes.diagramSettingsPane
        this.attributesPane.relayout()
        this.attributes.diagramSettingsPane.relayout()
    }
    this.attributes.diagramName.text=this.settings.diagramName
    this.attributes.diagramPath.text=this.settings.diagramPath
    this.attributes.rootStateName.text=rootState.name
  }
  
  Void saveAction()
  {
    this.attributes.diagramSave()
  }
  
  Void showStateAttributes()
  {
    JsmNode? activeNode:=this.stateMachineCanvas.currentNode
    
    if ( activeType!=ActiveType.STATE)
    {
        activeType=ActiveType.STATE
        this.attributesPane.content = this.attributes.statePane
        this.attributesPane.relayout()
        this.attributes.statePane.relayout()
    }
    
	  JsmSmNode activeState := activeNode
    if ( activeNode.type == NodeType.STATE )      
    {
	    this.attributes.displayStateAttributes((JsmState)activeState);
      this.attributes.entryActivity.enabled=true
      this.attributes.exitActivity.enabled=true
    }
    if ( activeNode.type == NodeType.FINAL )      
    {
	    this.attributes.displayStateAttributes((JsmState)activeState);
      this.attributes.entryActivity.enabled=true
      this.attributes.exitActivity.enabled=true
    }
    else if ( this.isPseudoState(activeNode))
    {
	    this.attributes.displayPseudoStateAttributes((JsmPseudoState)activeState);
    }
    
    if ( activeState.parentState != null )
    {
        this.attributes.parentState.text=activeState.parentState.name
    }
    else
    {
        this.attributes.parentState.text="None"
    }
    if ( activeState.spec != null )
    {
        this.attributes.internalDetails.text=activeState.spec
    }
    else
    {
        this.attributes.internalDetails.text=""
    }
  }
  
  Void showConnAttributes()
  {
      JsmConnection activeConn := this.stateMachineCanvas.selectedConns.first()
      if ( activeType!=ActiveType.CONN)
      {
        activeType=ActiveType.CONN
        this.attributesPane.content = this.attributes.transitionPane
        this.attributesPane.relayout()
        this.attributes.transitionPane.relayout()
      }
      this.attributes.displayConnAttributes(activeConn)

  }
  
  
  Void incSave()
  {
    this.attributes.incSave();
    this.gui.undoButton.enabled=true;
    this.gui.redoButton.enabled=false;
  }
  

}
