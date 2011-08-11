using gfx
using fwt

// TODO: add initial state to composite state

class StateMachineCanvas : JsmCanvas
{
  virtual JsmState? currentState
  virtual JsmState? rootState


  //Color cornerColor:=Color.fromStr("#B0B0B0")

  
  new make(JsmGui gui,JsmDiagram diagram) : super(gui,diagram)
  {
    this.gui = gui
    rootState=JsmState.maker(0,diagram.settings.diagramName,0,0,0,0)
    rootState.firstRegion().isRootState=true
    echo("== ${rootState.firstRegion().details} ${rootState.firstRegion().isRootState}")
    this.rootNode=rootState
    

    //attributes.canvas = this
    
    nodeIds=[Int:JsmNode][:] // initialize global nodeIds lookup table
    nodes=JsmNode[,] 
    selectedNodes=JsmNode[,] 
    selectedConns=JsmConnection[,] 
   
//    JsmState s1:=rootNode.newState(nextNodeId(),10,10)
//    containerNodes.add(s1)
//    echo("Added new state ${s1.details}")
//    nodes.add(s1)
  }
  
  
  override Void restore(JsmState newRootState)
  {
    super.restore(newRootState)
    this.rootState=newRootState
    validate()
  }

  override Void validate()
  {
    //echo("------------------------------------------------------------------")
    //echo("----Start Validate $rootState ----")
    //echo("------------------------------------------------------------------")
    rootState.validate()
    if ( this.nodes.size != rootState.getAllChildren.size )
    {
      echo("[error] Root state has ${rootState.getAllChildren.size} nodes, canvas has ${this.nodes.size} nodes")
      rootState.getAllChildren.each
      {
        echo("State: $it.name")
      }
      this.nodes.each
      {
        echo("state: $it.name")
      }
    }
    nodes.each 
    {   
      if ( it.parent == null )
      {
         echo("[error] Node ${it.name} has null parent")
      }
    }
    //echo("------------------------------------------------------------------")
    //echo("----End Validate----")
    //echo("------------------------------------------------------------------")
  }
  

  override Void deleteNode(JsmNode n)
  {
    nodes.remove(n)
    this.nodeIds.remove(n.nodeId)
    if ( n.type == NodeType.STATE )
    {
      containerNodes.remove(n)
    }
    n.parent.removeChild(n)
    
    // remove node from the list of container nodes
    if ( n.type == NodeType.STATE )
    {
      echo("deleting node $n.name from ")
      containerNodes.remove(n)
    }
    this.currentNode=null;
  }


  
  override Bool addNodeMode()
  {
    if ( mode == EditMode.ADD_STATE    ||
         mode == EditMode.ADD_INITIAL  ||
         mode == EditMode.ADD_FINAL    ||
         mode == EditMode.ADD_JOIN     ||
         mode == EditMode.ADD_FORK     ||
         mode == EditMode.ADD_CHOICE   ||
         mode == EditMode.ADD_JUNCTION  )
    {
      return(true)  
    }
    else
    {
      return(false)  
    }
  }
  
  override JsmNode? addNewNode(Event event)
  {
    this.newNode=null
    
    // add a node to the currently selected node 
    JsmState? targetNode:=this.currentNode
    // target node is root state if no node is selected
    if ( targetNode == null )
    {
      targetNode=rootState
      echo("**** targetNode is rootState")
    }
    else
    {
      echo("**** targetNode is ${targetNode.details}")
    }
    
    if ( targetNode.type != NodeType.STATE )
    {
      echo("ERROR - You can only add a node to a state")
      return(null)
    }
    
    if ( mode == EditMode.ADD_STATE)
    {
      if ( targetNode.isDefaultSize && targetNode != this.rootState )
      {
        echo("new state to state not enlarged") 
      }
      else
      {
        echo("Adding new state")
        this.newNode=targetNode.newState(nextNodeId(),event.pos.x,event.pos.y)
        echo("Added new state $newNode.name")
        containerNodes.add(this.newNode)
        this.diagram.redrawReason="mouse down add new state"
      }
    }
    else if ( mode == EditMode.ADD_INITIAL)
    {
      this.newNode=targetNode.addInitial(nextNodeId(),event.pos.x,event.pos.y)
      if ( newNode != null )
      {
        this.diagram.redrawReason="mouse down add initial"
        this.diagram.setMode(EditMode.ENTER_CONNECT)
      }
      else
      {
         Dialog.openErr(event.window, "$targetNode.name already has an initial state")
      }
    }
    else if ( mode == EditMode.ADD_FINAL)
    {
      this.newNode=targetNode.addFinal(nextNodeId(),event.pos.x,event.pos.y)
      if ( newNode != null )
      {
        this.diagram.redrawReason="mouse down add final"
        this.diagram.setMode(EditMode.ENTER_CONNECT)
      }
    }
    else if ( mode == EditMode.ADD_JOIN)
    {
      this.newNode=targetNode.addJoin(nextNodeId(),event.pos.x,event.pos.y)
      if ( newNode != null )
      {
        this.diagram.redrawReason="mouse down add Join"
        this.diagram.setMode(EditMode.ENTER_CONNECT)
      }
    }
    else if ( mode == EditMode.ADD_FORK)
    {
      this.newNode=targetNode.addFork(nextNodeId(),event.pos.x,event.pos.y)
      if ( newNode != null )
      {
        this.diagram.redrawReason="mouse down add FORK"
        this.diagram.setMode(EditMode.ENTER_CONNECT)
      }
    }
    else if ( mode == EditMode.ADD_CHOICE)
    {
      this.newNode=targetNode.addChoice(nextNodeId(),event.pos.x,event.pos.y)
      if ( newNode != null )
      {
        this.diagram.redrawReason="mouse down add CHOICE"
        this.diagram.setMode(EditMode.ENTER_CONNECT)
      }
    }
    else if ( mode == EditMode.ADD_JUNCTION)
    {
      this.newNode=targetNode.addJunction(nextNodeId(),event.pos.x,event.pos.y)
      if ( newNode != null )
      {
        this.diagram.redrawReason="mouse down add JUNCTION"
        this.diagram.setMode(EditMode.ENTER_CONNECT)
      }
    }
    if ( newNode != null )
    {
      echo("Added new node ${this.newNode.nodeId} ${this.newNode.details}")
      this.diagram.incSave();
    }
    return(newNode)
  }
  
  JsmRegion? findRegionContainingNode(JsmNode n1)
  {
    JsmRegion? newParentRegion:=rootState.firstRegion      
    Bool? stop:=null // null means do not stop, continue the iteration
    //selectedNodes.eachWhile |n2| // these are ordered by size, stop when n1 has new parent 
    //echo("-------------------")
    containerNodes.eachrWhile |n2| // these are ordered by size, stop when n1 >= n2 in size
    {
      //echo("n2=$n2.name, n1=$n1.name")
      if ( n1.isSmallerThan(n2) ) // otherwise n1 could not fit in n2
      {
	      if ( n2.type == NodeType.STATE )
	      {
	        JsmState s:=n2 // cast n2 as a state
	        JsmRegion? region:=s.findRegionContainingNode(n1) // see if n1 is in a region of n2
          if ( region != null )
          {
  	        newParentRegion=region
            //echo("$n2.name region($newParentRegion.name) is possible new parent of $n1.name")
          }
          else
          {
            //echo("$n1.name not in a region of $n2.name")
          }
	        //if ( newParentRegion != null ) // if n1 is in a region of n2 then this is the new parent
	        //{
	        //  stop=true
	        //}
	      }
        else
        {
          //echo("$n2.name is not new parent of $n1.name")
        }
      }
      else
      {
        //echo("stop checking for $n1.name >= $n2.name ")
        stop=true
      }
      return(stop)
    }
    //echo("$n1.name new parent region is $newParentRegion.name")
    //if ( newParentRegion == null )
    //{
    //}
    return(newParentRegion)
  }
  



}
