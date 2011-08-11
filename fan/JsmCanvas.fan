using gfx
using fwt

class JsmCanvas : Canvas
{
  EditMode mode := EditMode.ARROW
  virtual JsmState[] containerNodes
  virtual JsmNode[] nodes
  virtual [Int:JsmNode] nodeIds
  virtual JsmRegion? selectedRegion
  virtual JsmNode[] selectedNodes
  virtual JsmConnection[] selectedConns
  virtual JsmNode? currentNode
//  virtual JsmState? currentState
  virtual JsmState? rootNode
  @Transient JsmNode? newNode
  @Transient JsmNode? lastNodeAdded
  @Transient Str errorMsg:=""
  JsmGui? gui
  JsmDiagram? diagram
  JsmOptions? options
  Int startX := 0
  Int startY := 0
  Int origX := 0
  Int origY := 0
  Int endX := -1
  Int endY := -1
  Int nextNode:=0

  //Color cornerColor:=Color.fromStr("#B0B0B0")

  
  new make(JsmGui gui,JsmDiagram diagram)
  {
    this.diagram=diagram
    this.gui = gui
    
    nodeIds=[Int:JsmNode][:]
    nodes=JsmNode[,] 
    containerNodes=JsmNode[,] 
    selectedNodes=JsmNode[,] 
    selectedConns=JsmConnection[,] 
   
    d := |e| { dump(e) }
    mouseMove := |e| { evMouseMove(e) }
    mouseDown := |e| { evMouseDown(e) }
    mouseUp := |e| { evMouseUp(e) }
    keyDown := |e| { evKeyDown(e) }
    onFocus.add(d)
    onBlur.add(d)
    onKeyUp.add(d)
    onKeyDown.add(d)
    onKeyDown.add(keyDown)
    onMouseUp.add(mouseUp)
    onMouseDown.add(mouseDown)
    onMouseEnter.add(d)
    onMouseExit.add(d)
    onMouseMove.add(mouseMove)
    //onMouseMove.add(d)
    onMouseHover.add(d)
    onMouseWheel.add(d)
  }
  
  
  virtual Void restore(JsmState newrootNode)
  {
    rootNode=newrootNode
    nodeIds.clear
    rootNode.getAllSubstates()
    rootNode.restoreParentage(nodeIds,null)
    rootNode.restoreConnections(nodeIds)
    containerNodes=rootNode.getAllSubstates()
    nodeIds.remove(rootNode.nodeId)
    nodes=nodeIds.vals
    containerNodes.each { echo("Restored state $it $it.name") }
    nodes.each { echo("Restored node $it") }
    echo("+++")
    echo("Restored ${containerNodes.size} states for $rootNode.name($rootNode) $rootNode.getAllChildren.size nodes")
    this.nextNode=nodes.size
     
    selectedNodes.clear()
    selectedConns.clear()
    currentNode=null
//    currentState=null
    startX = 0
    startY = 0
    origX = 0
    origY = 0
    endX = -1
    endY = -1
  }


  override Size prefSize(Hints hints := Hints.defVal) { return Size.make(600, 600) }

  JsmNode? findNodeToSelect(Event event)
  {
    return(rootNode->findNodeToSelect(event.pos.x,event.pos.y))
  }
  
  JsmConnection[]? findConnToSelect(Event event)
  {
    return(rootNode->findConnToSelect(event.pos.x,event.pos.y))
  }
  
  Bool performAlign(AlignMode alignMode)
  {
    Bool moved:=false
     
    if ( selectedNodes.size < 2 )
    {
      return(false)
    }
    
    switch ( alignMode)
    {
      case AlignMode.CENTER:
          moved=performCenterAlign(); 
      case AlignMode.MIDDLE:
          moved=performMiddleAlign(); 
      case AlignMode.RIGHT:
          moved=performRightAlign(); 
      case AlignMode.LEFT:
          moved=performLeftAlign(); 
      case AlignMode.TOP:
          moved=performTopAlign(); 
      case AlignMode.BOTTOM:
          moved=performBottomAlign(); 
    }
    return(moved)
  }
  
  Bool performCenterAlign()
  {
    Bool moved:=false
    Int minX:=99999
    Int maxX:=0
    selectedNodes.each 
    { 
      if ( it.x1 < minX )
      {
        minX=it.x1
      } 
      if ( it.x2 > maxX )
      {
        maxX=it.x2
      } 
    }
    Int newX:=(minX+maxX)/2
    Int moveX:=0
    Int oldX:=0
    selectedNodes.each 
    { 
      oldX=(it.x1+it.x2)/2
      moveX=newX - oldX
      if ( moveX != 0 )
      {
        moved=true
      }
      it.x1+=moveX
      it.x2+=moveX
    }
    return(moved)
  }
  
  Bool performLeftAlign()
  {
    Bool moved:=false
    Int minX:=99999
    Int maxX:=0
    selectedNodes.each 
    { 
      if ( it.x1 < minX )
      {
        minX=it.x1
      } 
      if ( it.x1 > maxX )
      {
        maxX=it.x1
      } 
    }
    Int newX:=(minX+maxX)/2
    Int moveX:=0
    selectedNodes.each 
    { 
      moveX=newX - it.x1
      if ( moveX != 0 )
      {
        moved=true
      }
      it.x1+=moveX
      it.x2+=moveX
    }
    return(moved)
  }
  
  
  Bool performRightAlign()
  {
    Bool moved:=false
    Int minX:=99999
    Int maxX:=0
    selectedNodes.each 
    { 
      if ( it.x2 < minX )
      {
        minX=it.x2
      } 
      if ( it.x2 > maxX )
      {
        maxX=it.x2
      } 
    }
    Int newX:=(minX+maxX)/2
    Int moveX:=0
    selectedNodes.each 
    { 
      moveX=newX - it.x2
      if ( moveX != 0 )
      {
        moved=true
      }
      it.x1+=moveX
      it.x2+=moveX
    }
    return(moved)
  }
  
  Bool performMiddleAlign()
  {
    Bool moved:=false
    Int minY:=99999
    Int maxY:=0
    selectedNodes.each 
    { 
      if ( it.y1 < minY )
      {
        minY=it.y1
      } 
      if ( it.y2 > maxY )
      {
        maxY=it.y2
      } 
    }
    Int newY:=(minY+maxY)/2
    Int moveY:=0
    Int oldY:=0
    selectedNodes.each 
    { 
      oldY=(it.y1+it.y2)/2
      moveY=newY - oldY
      it.y1+=moveY
      it.y2+=moveY
    }
    return(moved)
  }
  
  Bool performTopAlign()
  {
    Bool moved:=false
    Int minY:=99999
    Int maxY:=0
    selectedNodes.each 
    { 
      if ( it.y1 < minY )
      {
        minY=it.y1
      } 
      if ( it.y1 > maxY )
      {
        maxY=it.y1
      } 
    }
    Int newY:=(minY+maxY)/2
    Int moveY:=0
    selectedNodes.each 
    { 
      moveY=newY - it.y1
      if ( moveY != 0 )
      {
        moved=true
      }
      it.y1+=moveY
      it.y2+=moveY
    }
    return(moved)
  }
  
  
  Bool performBottomAlign()
  {
    Bool moved:=false
    Int minY:=99999
    Int maxY:=0
    selectedNodes.each 
    { 
      if ( it.y2 < minY )
      {
        minY=it.y2
      } 
      if ( it.y2 > maxY )
      {
        maxY=it.y2
      } 
    }
    Int newY:=(minY+maxY)/2
    Int moveY:=0
    selectedNodes.each 
    { 
      moveY=newY - it.y2
      if ( moveY != 0 )
      {
        moved=true
      }
      it.y1+=moveY
      it.y2+=moveY
    }
    return(moved)
  }
  
  Void setSelectedNodes()
  {
    Int areaX1:=startX
    Int areaX2:=endX
    Int areaY1:=startY
    Int areaY2:=endY
    if ( endX < startX )
    {
      areaX1=endX
      areaX2=startX
    }
    if ( endY < startY )
    {
      areaY1=endY
      areaY2=startY
    }
    deselectNodes
    echo("2--------------clear----------------")
    // we need to set the current node to one of the nodes since we will use that for resizing
    // 
    //containerNodes.eachrWhile  |state|
    //containerNodes.each  |state|
    nodes.each  |state|
    { 
      if ( state->inArea(areaX1,areaY1,areaX2,areaY2) == true )
      {
        echo("Add1) it.name")
        selectedNodes.add(state) // ordered by size since nodes is ordered by size
        state.hasFocus=true
    //    return(state) // break out of loop
      } 
      else
      {
        //state.hasFocus=false
    //    return(null)  // continue loop
      }
    }
  }
  
  Void deselectNodes()
  {
    selectedNodes.each 
    { 
      echo("deselecting $it.name")
      it.hasFocus=false
    }
    selectedNodes.clear 
    this.currentNode=this.rootNode
  }
  
  Void deselectConns()
  {
    selectedConns.each 
    { 
      it.selected=false
    }
    selectedConns.clear 
  }
  
  ** 
  ** Mouse Down Event
  ** 
  Void evKeyDown(Event event)
  {
    switch (event.key)
    {
      case Key.delete:
      case Key.backspace: 
        //echo("delete");
        if ( deleteSelectedConns() || deleteSelectedNodes() )
        {
          this.diagram.redrawReason="delete operation"
        }
      default: 
        //echo("ignore key")
    }
    this.diagram.checkRedraw();
    
    //echo("Key down - mode is ${event}")
  }
  
  Bool deleteSelectedNodes()
  {
    if ( selectedNodes.size == 0)
    {
      //echo("No nodes selected for deletion")
      return(false);
    }
    selectedNodes.each
    {
      deleteNode(it)
    }
    nodes.each
    {
     it.remove(selectedNodes)
    }
    selectedNodes.each
    {
      it.parent.removeChild(it)
    }
    deselectNodes; 
    this.currentNode=this.rootNode;

    return(true)
  }
  
  
  virtual Void deleteNode(JsmNode n)
  {
    echo("deleteing node $n.name")
    nodes.remove(n)
    n.parent.removeChild(n)
    
    // remove node from the list of container nodes
    if ( n.type == NodeType.STATE )
    {
      echo("deleteing node $n.name from ")
      containerNodes.remove(n)
    }
  }

  
  Bool deleteSelectedConns()
  {
    if ( selectedConns.size == 0)
    {
      //echo("No connections selected for deletion")
      return(false);
    }
    selectedConns.each
    {
      it.remove()
    }
    selectedConns.clear()
    return(true)
  }
  
  ** 
  ** Mouse Down Event
  ** 
  virtual Void evMouseDown(Event event)
  {
    this.diagram.redrawReason=null
    changeSelection(event) // selectedNodes will remain unchanged unless a conn is selected
    
    echo("Mouse down - mode is ${mode}")
    // you cannot add a node if the target is too small
    if ( this.selectedRegion != null  )
    { 
      echo ("Starting region move")
      startRegionMove(event)
    }
    else if ( addNodeMode )
    {
      JsmNode? targetNode:=findNodeToSelect(event)
      // add a new node if there is room
      echo ("target node default $targetNode.isDefaultSize")
      if ( targetNode != this.rootNode && targetNode != null && targetNode.isDefaultSize == true )
      {
        startMouseDown(event)
        this.diagram.setMode(EditMode.ARROW)
      }
      else
      {
        JsmNode? mynewNode:=addNewNode(event)
        addedNode(mynewNode)
        
      }
    }
    else if ( currentNode != null  && currentNode != this.rootNode)
    { 
      startMouseDown(event)
    }
    //else if ( this.selectedConns.size > 0 )
    //{
    //  echo("redraw connections")
    //  //this.redraw("redrawing connections")
    //}
    else // start drawing mouse selection region
    {
      echo("Mode is $mode -- else for mouse down $diagram.mode")
      if ( this.diagram.mode == EditMode.CONNECT || this.diagram.mode == EditMode.ENTER_CONNECT )
      {
        echo("chaning to pointer")
        this.diagram.setMode(EditMode.ARROW)
      }
      startSelectionRegion(event)
    }
    echo("Mode is $mode")
    this.diagram.checkRedraw()
  }
  
  
  
  Void startRegionMove(Event event)
  {
  }
  
  Void endRegionMove(Event event)
  {
    if ( selectedRegion.regionNotMoved )
    {
      // nothing to do
      selectedRegion.hasFocus=false
      selectedRegion=null
    }
    else if ( selectedRegion.finishRegionMove() )
    {
      reparentNodes()
      selectedRegion=null
    }
    else
    {
      selectedRegion=null
    }
  }
  
  
  
  
  Void startMouseDown(Event event)
  {
    if ( mode == EditMode.CONNECT )
    {
       setCurrentNode(currentNode) // this will not deselect other nodes
      echo("startMouseDown - starting connection")
       currentNode.startConnection() 
    }
    else 
    {
      Corner c := currentNode->getCorner(event.  pos.x,event.pos.y) 
      if ( c != Corner.NOT_CORNER ) // we selected a corner for resizing
      { 
        // are we in a corder of a node
        origX=currentNode.getCornerX(c)
        origY=currentNode.getCornerY(c)
        this.diagram.setMode(EditMode.RESIZE)
        deselectChildren() // resizing a parent does not resize a child, just peers or children of peers
        selectedNodes.each { it.currentCorner = c }
      } 
      else 
      {
        //newNode:=checkChangedSelectedNodes()
        // 
        if ( selectedNodes.contains(currentNode.parentNode) )
        {
          newNode:=currentNode
          deselectNodes()
          //echo("1--------------clear----------------")
          echo("startMouseDown Selecting node $newNode.name")
          this.setCurrentNode(newNode)
          //echo("Select $currentNode.name - now add children")
          selectChildren(currentNode) // only states can have children
          newNode.hasFocus=true
          
          this.diagram.redrawReason="mouse down new Node"
        }
        else
        {
          selectChildren(currentNode) // move parent and children
        }
        //echo("Selected:")
        //selectedNodes.each{echo("  $it.name")}
        //echo("=========")
        startX=event.pos.x
        startY=event.pos.y
        origX=event.pos.x
        origY=event.pos.y
        endX=event.pos.x
        endY=event.pos.y
        mode=EditMode.MODE_MOVE
      }
    }
  }
  
  virtual Void checkChangedSelectedNodes()
  {
    // after selecting a set of nodes a user may select 
    // one of those nodes and move all of the selected nodes
    // But if only a parent node is selected and we click
    // on a child node we want to deselect the parent and 
    // any other nodes selected and select only the child
    // node. 
    
    // not the root node
    if ( currentNode != null && currentNode.parentNode != null )
    {
      echo("currentNode=$currentNode.name")
      //echo("currentState=$currentState.name")
      echo("selectedNodes=$selectedNodes.size")
      if ( selectedNodes.contains(currentNode.parentNode) )
      {
        //echo("Clicked on child of selected node")
        newNode:=currentNode
        deselectNodes() 
        this.setCurrentNode(newNode)
      }
      else
      {
        //echo("Clicked on state whose parent is not already selected")
      }
    }
    else
    {
      deselectNodes() 
    }
  }

  
//  // this is only called when a new selection of a connection or node is 
//  // possibly being made
//  virtual Void OLDchangeSelection(Event event)
//  {
//    echo("Change selection...")
//    JsmNode? newlySelectedNode:=findNodeToSelect(event)
//    selectedRegion=null
//    if ( newlySelectedNode != null && selectedNodes.contains(newlySelectedNode)  )
//    {
//      // do not deselect the other selected nodes on mouse down - it may be start of a move
//      currentNode=newlySelectedNode     
//      if ( currentNode == this.rootNode)
//      {
//        currentNode=null
//      }
//      echo("*** Not Changing selection... $newlySelectedNode.name")
////      if ( currentNode.type == NodeType.STATE)
////      {
////        currentState=currentNode
////      }
////      else
////      {
////        currentState=null
////      }
//    }
//    else
//    {
//      echo("Changing selection... $newlySelectedNode.name")
//      setCurrentNode(newlySelectedNode)
//      if ( newlySelectedNode != null && newlySelectedNode != this.rootNode )
//      {
//        deselectConns
//      }
//      else
//      {
//        selectConnection(event)
//      }
//    }
//    if ( currentNode != null && currentNode.type == NodeType.STATE )
//    {
//      JsmState selectedState:=currentNode
//      selectedRegion=selectedState.regionSelected(event.pos.x,event.pos.y)
//      if ( selectedRegion != null)
//      {
//        selectedRegion.hasFocus=true
//        deselectNodes()
//        deselectConns()
//        mode=EditMode.MOVE_REGION
//        echo("Starting move of region dashed line") 
//      }
//    }
//  }
//  
 // this is only called when a new selection of a connection or node is 
  // possibly being made
  virtual Void changeSelection(Event event)
  {
    echo("Change selection...")
    deselectConns
    selectConnection(event)
    if ( selectedConns.size > 0 )
    {
      echo("deselecting nodes $selectedNodes.size")
      this.deselectNodes()
      return
    }
    
    JsmNode? newlySelectedNode:=findNodeToSelect(event)
    selectedRegion=null
    if ( newlySelectedNode != null && selectedNodes.contains(newlySelectedNode)  )
    {
      // do not deselect the other selected nodes on mouse down - it may be start of a move
      currentNode=newlySelectedNode     
//      if ( currentNode == this.rootNode)
//      {
//        currentNode=null
//      }
      echo("*** Not Changing selection... $newlySelectedNode.name")
    }
    else
    {
      echo("Changing selection... $newlySelectedNode.name")
      setCurrentNode(newlySelectedNode)
//      if ( newlySelectedNode != null && newlySelectedNode != this.rootNode )
//      {
//        deselectConns
//      }
//      else
//      {
//        selectConnection(event)
//      }
    }
    
    // Handle start of region dashed line move
    if ( currentNode != this.rootNode && currentNode.type == NodeType.STATE )
    {
      JsmState selectedState:=currentNode
      selectedRegion=selectedState.regionSelected(event.pos.x,event.pos.y)
      if ( selectedRegion != null)
      {
        selectedRegion.hasFocus=true
        deselectNodes()
        deselectConns()
        mode=EditMode.MOVE_REGION
        echo("Starting move of region dashed line") 
      }
    }
  }
    
  
  Void startSelectionRegion(Event event)
  {
      if ( selectedNodes.size > 0 )
      {
        deselectNodes()
        this.diagram.redrawReason="mouse down deselect"
      }
      //echo("MouseDown Mode=SELECT")
      startX=event.pos.x
      startY=event.pos.y
      endX=event.pos.x
      endY=event.pos.y
      mode=EditMode.SELECT
      //echo("Mode = SELECT")
  }
  
  virtual Bool addNodeMode()
  {
     // override with logic here 
     return(false);
  }
  
  virtual JsmNode? addNewNode(Event event)
  {
    // override logic here 
    return(null)
  }
  
  Void addedNode(JsmNode? newNode)
  {
    echo("Enter addedNode - ${mode}")
    if ( newNode != null )
    {
      setCurrentNode(newNode)
      if ( this.nodesIntersecting == true )
      {
        echo("Cannot add intersecting node - deleting $newNode.name")
        this.deleteNode(newNode) 
        echo("after deleting $newNode.name")
        setCurrentNode(this.rootNode)
      }
      else
      {
        echo("Adding node ${newNode.nodeId} ${newNode.details}")
        nodes.each 
        {   
          if ( it.name == newNode.name )   
          {
            this.reportError("Internal Error - $newNode.name id=$newNode.nodeId already exists as ${it.nodeId}" )
          }
        }
        if ( nodeIds.containsKey(newNode.nodeId))
        {
          reportError("Internal Error - $newNode.name id=$newNode.nodeId already exists as ${nodeIds[newNode.nodeId].name}" )
        }
        else
        {
          nodes.add(newNode)
          nodeIds.add(newNode.nodeId,newNode)
          echo("Adding new node $newNode.nodeId / $nodes.size to canvas")
          orderNodesBySize()
          this.lastNodeAdded=this.newNode
          this.newNode=null
        }
      }
    }
    

  }
  Void reportError(Str errMsg)
  {
    this.errorMsg+="errMsg"
  }
  
  
  Int nextNodeId()
  {
    return(++nextNode);
  }
  
  Void selectConnection(Event event)
  {
    selectedConns.each
    {
        it.selected=false
    }
//    if ( selectedNodes.size > 0 )
//    {
//      echo("already selected node - not searching for connection")
//      return;
//    }
    
    JsmConnection[]? newlySelectedConn:=findConnToSelect(event)
    if ( newlySelectedConn.size == 1 )
      {
      echo("selecting connection...")
      newlySelectedConn.first.selected=true
      selectedConns=newlySelectedConn
    }
    else if ( newlySelectedConn.size == 0 )
    {
      echo("no connections selected...")
      //this.deselectConns()
        
    }
    else
    {
      echo("multiple connections selected...")
    }
    
  }
  

  Void selectChildren(JsmNode parentNode)
  {
    JsmNode[] childNodes:=parentNode.getAllChildren()
    childNodes.each { selectedNodes.remove(it) }
    childNodes.each
    {
      //echo("Adding child $it.name")
      selectedNodes.add(it)
    }
    selectedNodes.sort |JsmNode a, JsmNode b->Int| { return (a.x2 - a.x1) <=> (b.x2 - b.x1) }
    //echo("^^^^^^^^^^^^^^^^^^^^")
  }
  
  Void deselectChildren()
  {
    JsmNode[] childNodes:=[,]
    selectedNodes.each { childNodes.addAll(it.getAllChildren()) }
    childNodes.each
    {
      //echo("Removing child $it.name")
      selectedNodes.remove(it)
      it.hasFocus=false
    }
    //echo("^^^^^^^^^^^^^^^^^^^^")
  }
  
  
  ** 
  ** Mouse Up Event
  ** 
  Void evMouseUp(Event event)
  {
    
    if ( currentNode == null )
    {
      echo("================= Mouse Up $mode -- null currentNode")
    }
    else
    {
      echo("================= Mouse Up $mode -- $currentNode.name")
    }
    
    if ( mode == EditMode.ENTER_CONNECT )
    {
      mode=EditMode.CONNECT
      this.setCurrentNode(this.rootNode)
    }
    else if ( mode == EditMode.CONNECT )
    {
      if ( currentNode != null && currentNode != this.rootNode )
      {
        JsmNode? targetNode:=findNodeToSelect(event)
        if ( targetNode != null && targetNode != currentNode && targetNode != this.rootNode )
        {
          // make a transition between Nodes  
          sourceNode:=currentNode
          echo("---End Connection---------- to $targetNode.name")
          JsmConnection? newConn:=currentNode.endConnection(targetNode)
          deselectNodes()
          deselectConns()
          if ( newConn != null)
          {
            newConn.selected=true
            selectedConns.add(newConn)
            this.diagram.incSave();
          }
          else
          {
            checkErrorMsg()
          }
          echo("end connection - selected conns $selectedConns.size $sourceNode.name,$targetNode.name")
          sourceNode.pendingConnection(0,0) 
          setCurrentNode(this.rootNode)
        }
        else if ( targetNode == currentNode )
        {
          echo("target of connection is current node $targetNode.name")
          // nothing to do 
          this.diagram.setEditMode(EditMode.ARROW)
          currentNode.pendingConnection(0,0) 
        }
        else
        {
          currentNode.pendingConnection(0,0) 
          this.diagram.setEditMode(EditMode.ARROW)
          setCurrentNode(this.rootNode)
        }
//        currentState=null
      }
      //echo("Mode=CONNECT")
      this.diagram.redrawReason="mouse up connect"
    }
    else if ( mode == EditMode.MOVE_REGION )
    {
      mode=EditMode.ARROW
      echo("end region move Mode=ARROW")
      this.diagram.redrawReason="mouse up select"
      this.endRegionMove(event)
    }
    else if ( mode == EditMode.SELECT )
    {
      mode=EditMode.ARROW
      echo("Mode=ARROW")
      this.diagram.redrawReason="mouse up select"
    }
    else if ( mode == EditMode.RESIZE )
    {
      finishMoveOrResize(event)
      //mode=EditMode.ARROW
      //echo("Mode=ARROW")
    }
    else if ( mode == EditMode.MODE_MOVE )
    {
      echo("Mode=MOVE")
      finishMoveOrResize(event)
      //mode=EditMode.ARROW
      //echo("Mode=ARROW")
    }
    else
    {
      echo("Mode=other? $mode")
    }
    this.diagram.checkRedraw
    //repaint()
  }
  
  Void checkErrorMsg()
  {
    if ( this.rootNode.errorMsg != "" || this.errorMsg != "" )
    {
      echo(Dialog.openErr(this.gui.mainWindow, this.rootNode.errorMsg+"\r\n"+this.errorMsg, ArgErr()))
      this.rootNode.errorMsg = ""
      this.errorMsg = ""
    }
  }
  
  ** 
  ** Mouse Move Event
  ** 
  Void evMouseMove(Event event)
  {
    //echo("mouse move $mode")
    if ( mode == EditMode.RESIZE )
    {
      resizeSelection(event.pos.x,event.pos.y)
    }
    else if ( mode == EditMode.SELECT )
    {
       endX=event.pos.x
       endY=event.pos.y
       setSelectedNodes()
       this.diagram.redrawReason="mouse move select"
    }
    else if ( mode == EditMode.MOVE_REGION )
    {
      endX=event.pos.x
      endY=event.pos.y
      selectedRegion.pendingMove(endX,endY)
      echo("Moving region")
      this.diagram.redrawReason="mouse move select"
    }
    else if ( mode == EditMode.CONNECT )
    {
      if ( currentNode != null && currentNode != this.rootNode )
      {
        echo("${currentNode.name}> connect from")
        currentNode.pendingConnection(event.pos.x,event.pos.y) 
        this.diagram.redrawReason="mouse move connect"
      }
      else
      {
        echo("mouse move on connect - current node is null")
      }
    }
    else if ( mode == EditMode.MODE_MOVE )
    {
       moveSelection(event.pos.x,event.pos.y)
    }
    else //AR
    {
       //changeSelection(event)        
    }
    //echo("----------------end mouse move---------------- -- mode=$mode")
    this.diagram.checkRedraw()
  }
  
  Bool validateMoveOrResize()
  {
    // only one initial state per  composite state
    Bool valid:=true
    nodes.each |n|
    {
//      if ( n.)
      
    }
    return(valid) 
  }
  
  Bool nodesIntersecting()
  {
    Bool nodeIntersects:=false
    JsmNode? node:=selectedNodes.eachWhile |n1|
    { 
      JsmNode? n:=this.nodes.eachWhile  |n2|
      {
        if ( n1 != n2 )
        {
          if ( mode == EditMode.RESIZE || ! selectedNodes.contains(n2) ) 
          {
            //  echo("nodes intersect0 $n1.name($n1.x1,$n1.y1,$n1.x2,$n1.y2) $n2.name($n2.x1,$n2.y1,$n2.x2,$n2.y2)")
            // do these nodes overlap either entirely or partially
            if ( n1.overlapsNode(n2))
            {
              if ( n1.containsNode(n2))
              {
                 // new container relationship                    
              }
              else if ( n2.containsNode(n1))
              {
                 // new container relationship                    
              }
              else
              {
                //echo("nodes intersect1")
                // partial overlap -- intersection
                return(n2) // intersecting node
              }
            }
          }
        }
        return(null) // not intersecting
      }
      return(n)
    }
    if ( node != null )
    {
        //echo("nodes intersect2")
        return(true) //
    }
    else
    {
       return(false) 
    }
    //return(nodeIntersects)
  }
  
  
  
  Void finishMoveOrResize(Event ev)
  {
    if ( selectedNodes.size == 0 || (ev.pos.x == this.origX && ev.pos.y == this.origY))
    {
      this.diagram.setMode(EditMode.ARROW)
      //this.deselectNodes()
      //this.deselectConns()
      if ( selectedNodes.size == 0 )
      {
        //echo("No nodes selected for move or resize -- $mode")
      }
      else
      {
        echo("Selected nodes did not move or resize -- $mode")
        changeSelection(ev)
        echo("Finished change selection $mode")
      }
    }
   //echo("nodes intersect3")
    // invalid move - move back to original position
    else if ( nodesIntersecting() == true )
    {
      if ( mode == EditMode.MODE_MOVE )
      {
        moveSelection(origX,origY)
        //echo("nodes intersect4")
      }
      else if ( mode == EditMode.RESIZE )
      {
        resizeSelection(origX,origY)
        //echo("nodes intersect5")
        this.diagram.redrawReason="resize intersect"
      }
    }
    else
    {
      echo("reparenting nodes after move or resize")
      reparentNodes(); 
      this.diagram.redrawReason="Changed parentage of node"
      this.diagram.incSave();
    }
    this.diagram.setMode(EditMode.ARROW)
    //this.cursor=Cursor.defVal
  }

  

  virtual Void reparentNodes()
  {
    //selectedNodes.each 
    nodes.each 
    {
      echo("Looking for new parent for $it.name")
      JsmRegion? newRegion := findNewContainingRegion(it)
      changeParentRegion(it,newRegion)
    }
  }
  
    
  JsmRegion? findNewContainingRegion(JsmNode n1)
  {
      Int w:=n1.x2 - n1.x1
      JsmRegion? newParentRegion:=null
      // containerNodes are ordered by larger to smaller size so the most nested one will be returned
      containerNodes.eachWhile |n2| 
      {
        Bool? stop:=null // null means do not stop, continue the iteration
        if ( n1 == n2 )
        {
          
        }
        else if ( (n2.x2 - n2.x1) <= w) // reached a node same size or smaller - cannot be container after this
        {
          echo("state $n2.name is smaller or equal to $n1.name")
          stop=true
        }
        else
        {
          newParentRegion=n2.findRegionContainingNode(n1)
          if ( newParentRegion != null )
          {
            stop=true
          }
          else
          {
            echo("state $n2.name does not contain node $n1.name")
          }
        }
        return(stop)
      }
      if ( newParentRegion == null )
      {
        newParentRegion=rootNode.firstRegion()
      }
    return(newParentRegion)
  }


    
  Void changeParentRegion(JsmNode n1,JsmRegion newParentRegion)
  {
      if ( newParentRegion != n1.parent )
      {
        echo("Reparenting node $n1.name")
        if ( n1.parent != null)
        {
          echo("--remove $n1.name from $n1.parent.name")
          n1.parent.removeChild(n1)
        }
        echo("--add $n1.name from $newParentRegion.name")
        newParentRegion.addChild(n1)
//        if ( n1 == currentNode )
//        {
//          setCurrentNode(currentNode)
//        }
      }
      else
      {
        echo("No change of parent for $n1.name - parent is still $n1.parent.name")
      }
  }
  
  Void resizeSelection(Int x,Int y)  
  {
     if ( selectedNodes.size > 0 )
     {
       Int diffX := x - this.currentNode.getCurrentCornerX()
       Int diffY := y - this.currentNode.getCurrentCornerY()
       selectedNodes.each 
       { 
         //echo("Resizing node $it.name")
         it.resize(diffX,diffY) 
         //it.move(x - startX,y - startY) 
       }
       orderNodesBySize()
       //startX=x
       //startY=y
       this.diagram.redrawReason="mouse move resize"
     }
  }
  
  // paint larger nodes first so that they do not obscure smaller nodes
  Void orderNodesBySize()
  {
    nodes.sort |JsmNode a, JsmNode b->Int| { return (a.x2 - a.x1) <=> (b.x2 - b.x1) }
    // order the states from larger to smaller
    containerNodes.sortr |JsmState a, JsmState b->Int| { return (a.x2 - a.x1) <=> (b.x2 - b.x1) }
    echo("---sorted descending by size---")
    containerNodes.each { echo("$it.name") }
  }
  
    
  Void moveSelection(Int x,Int y)  
  {
     if ( selectedNodes.size > 0 )
     {
       // confirm that no node intersects another node
       //echo("-------------")
       selectedNodes.each 
       { 
         //echo("Move $it.name $x - $startX, $y $startY")
         it.move(x - startX,y - startY) 
       }
       //echo("=============")
       startX=x
       startY=y
       selectedNodes.each
       {
         it.checkSwitchSides()  
       }
       this.diagram.redrawReason="move selection"
     }
  }
  

  Void setCurrentNode(JsmNode? node)
  {
    echo((">>>>>>>>>>>>>>>>>>>>>call deselectNodes"))
    deselectNodes()
    echo(("<<<<<<<<<<<<<<<<<<<<<call deselectNodes"))
    if ( node == rootNode )
    {
      echo("node is root node")
      currentNode=rootNode
      this.deselectNodes()
      //throw(Err("[error] You cannot set the rootNode as the current node"))
    }
    else if ( node != null )
    {  
      selectedNodes.add(node)  // first one added after clear
      if ( currentNode != node )
      {
        this.diagram.redrawReason="Changed current node"
      }
      echo("Setting focus for $node.name $this.mode") 
      currentNode=node
      node.hasFocus=true
//      if ( currentNode.type == NodeType.STATE )
//      {
//        currentState=node
//        if ( currentState.parent == null )
//        {
//          echo("[error] $currentState.name has a null parent") 
//        }
//        else
//        {
//          //echo("Current state is $currentState.name in region $currentState.parent.name")
//        }
//      }
//      else
//      {
//        //echo("Current node is not a state - $currentNode.name")
//      }
    }
    else
    {
      echo("node is null")
      currentNode=this.rootNode
//      currentState=null
    }
  }
  

  override Void onPaint(Graphics g)
  {
    // called by repaint
    w := size.w
    h := size.h
    
    g.brush = Color.white;
    g.fillRect(0, 0, w, h)

    
    g.brush = Color.black
    

//    Border("2 inset 3") 
    
   // g.border = Border("2 inset 3")
    
    //echo("$name> paint")
    
    g.brush = Color.black
    //nodes.each { it->calcConnections() }
    rootNode.calcConnections()
    //rootNode.draw(g)
    //echo("SMCanvas.draw -- containerNodes")
    //echo("SMCanvas.draw -- ------")
    //nodes.each { it->draw(g) }
    rootNode.draw(g)
    //echo("draw states")
    //containerNodes.each { echo("--draw $it.name $it.parentState.name") }
    containerNodes.each { it->draw(g) }
    rootNode.drawConnections(g)
     if ( mode == EditMode.SELECT && endX > 0 )
     {
       g.brush = Color.gray
       g.pen = Pen { width = 1; dash=[2,2].toImmutable }
       g.drawRect(startX,startY,endX - startX,endY - startY)
     }
  }
  
  Void redraw(Str reason)
  {
    this.diagram.updateAttributes()
    validate()
    selectedNodes.each 
    { 
      it.reorderSlots()
    }
    //echo("redraw> $reason")
    repaint
  }
  
  virtual Void validate()
  {

  }
  

  Void dump(Event event)
  {
    //if (event.id == EventId.focus || event.id == EventId.blur)
    //  redraw("dump")

    //echo("dump $name> $event")
  }

  Str? name
  JsmGui? demo
}

