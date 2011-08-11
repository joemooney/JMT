using gfx
using fwt

@Serializable
class JsmRegion
{
  Str name
  Int x1
  Int y1
  Int x2
  Int y2
  Int pendingX
  Int pendingY
  Int minWidth:=20
  Int minHeight:=20
  Bool horizontal:=true
  @Transient virtual JsmState? parent
  @Transient Bool hasFocus:=false
  
  virtual JsmNode[] children
  virtual Int y
  @Transient virtual JsmState[] states:=JsmState[,] 
  @Transient JsmState? parentState
  Bool isRootState:=false
  //@Transient override JsmNode? parent

  
  new make(|This| f)
  {
    f(this)
    echo("making a new region $name")
  }
   
  new maker(JsmState parent,Str name,Int x1,Int y1,Int x2,Int y2)
  {
    this.parent=parent
    minHeight=30
    children=JsmNode[,] 
    
    this.parent=parent
    this.name = name;
    this.x1=x1;
    this.y1=y1;
    //this.w=w;
    //this.h=h;
    //this.x2=x1+w;
    //this.y2=y1+h;
    this.x2=x2
    this.y2=y2
  }
  

  virtual JsmState[] getAllSubstates()
  {
    JsmState[] substates := JsmState[,]
    //echo("State.getSubstates: $this.name ")
    states.each 
    {   
      substates.add(it )
      substates.addAll(it.getAllSubstates)
    }
    //echo("getSubstates: $this.name -- $substates.size substate")
    return(substates)
  }
  
  ** When we save off a state hierachy we cannot save the pointers linking
  ** from the child nodes back up to their parents. We can only save in one 
  ** direction.
  ** This method goes through each of the states in the region and passes
  ** down the region reference which is the parent of the state 
  Void restoreParentage([Int:JsmNode] nodeIds,JsmState newParent )
  {
    echo("Restoring region parent $name ($this) to $newParent.name  children:$children.size")
    this.parentState=newParent
    this.parent=newParent
    children.each 
    {   
      it.restoreParentage(nodeIds,this)
      if ( it.type == NodeType.STATE )
      {
        states.add(it)
        echo("Region $this.name now has $states.size states -- added $it.name")
      }
    }
  }
  
  virtual Void restoreConnections([Int:JsmNode] nodeIds)
  {
    echo("Restoring connections for $name ($this)")
    children.each 
    {   
      it.restoreConnections(nodeIds)
    }
  }

  virtual Void calcConnections()
  {
    children.each
    {
      it.calcConnections()
    }
  }

  virtual JsmNode[] getAllChildren()
  {
    JsmNode[] descendents := children.dup
    //echo("Container.getAllChildren: $this.name")
    children.each 
    {   
      descendents.addAll(it.getAllChildren)  
    }
    return(descendents)
  }
  
  Bool closeToRegion(Int x,Int y)
  {
    if ( this.horizontal )
    {
      if ( JsmUtil.closeToLine(x1,y1,x2,y1,x,y ) )
      {
        return(true)
      }
      else
      {
        return(false)
      }
    }
    else if ( JsmUtil.closeToLine(x1,y1,x1,y2,x,y ) )
    {
      return(true)
    }
    else
    {
      return(false)
    }
  }

  Void draw(Graphics g)
  {
    children.each
    {
      //echo("Region.draw child $it.name")
      it.draw(g)
    }
    if ( this.parent.firstRegion != this )
    {
      //echo("Draw horizontal region separator")
      
      Pen oldPen:=g.pen
      g.pen = Pen { width = 1; dash=[4,2].toImmutable }
      if ( this.hasFocus )
      {
        g.brush = Color.orange
      }
      else
      {
        g.brush = Color.black
      }
      
      if ( this.horizontal )
      {
        if ( this.pendingY > 0 )
        {
          g.drawLine(x1, pendingY, x2, pendingY)
        }
        else
        {
          g.drawLine(x1, y1, x2, y1)
        }
      }
      else
      {
        if ( this.pendingX > 0 )
        {
          g.drawLine(pendingX, y1, pendingX, y2)
        }
        else
        {
          g.drawLine(x1, y1, x1, y2)
        }
      }
      g.pen = oldPen
    }
  }

  ** Check if node intersects the region dashed line
  Bool intersectsRegion(JsmNode n)
  {
    if ( pendingX > 0 || pendingY > 0 )
    {
      if ( horizontal)
      {
        if ( pendingY >= n.y1 && pendingY <= n.y2 )
        {
          return(true)          
        }
        else
        {
          return(false)          
        }
      }
      else
      {
        if ( pendingX >= n.x1 && pendingX <= n.x2 )
        {
          return(true)          
        }
        else
        {
          return(false)          
        }
      }
    }
    else if ( (x1 >= n.x1 && x1 <= n.x2) || (y1 >= n.y1 && y1 <= n.y2) )
    {
      echo("Region $name($x1,$y1) intersects $n.name ($n.x1,$n.y1,$n.x2,$n.y2)")
      return(true)
    }
    else
    {
      //echo("Region $r.name does not intersect $n.name")
      //echo("Region $r.x1,$n.x1 $r.x2,$n.x2 $r.y1,$n.y1 $r.y2,$n.y2 ")
      return(false) // null will cause the loop to continue to the next region
    }
  }
  
  ** Check if region dashed line has moved
  virtual Bool regionNotMoved()
  {
    if ( horizontal )
    {
      if ( pendingY == 0 || pendingY == x1)
      {
        pendingX=0
        pendingY=0
        return(true)
      }
      else
      {
        return(false)
      }
    }
    else
    {
      if ( pendingX == 0 || pendingX == x1)
      {
        pendingX=0
        pendingY=0
        return(true)
      }
      else
      {
        return(false)
      }
    }
  }
  
  virtual Bool finishRegionMove()
  {
    JsmNode[] nodesToCheck:=this.parent.getImmediateChildren
    Bool? intersects:=nodesToCheck.eachWhile |n|
    {   
      echo("checking $n.name intersects region $name")
      if ( intersectsRegion(n) )  
      {
        echo("yes $n.name intersects region $name")
        return(true)
      }
      else
      {
        return(null)
      }
    }
    this.hasFocus=false
    if ( intersects == null )
    {
      JsmRegion? prevRegion:=this.parent.previousRegion(this)
      if ( horizontal )
      {
        this.y1 = this.pendingY 
        if ( prevRegion != null )
        {
          prevRegion.y2=this.y1
        }
      }
      else
      {
        this.x1 = this.pendingX 
        if ( prevRegion != null )
        {
          prevRegion.x2=this.x1
        }
      }
      this.pendingX = 0
      this.pendingY = 0
      return(true)
    }
    else
    {
      this.pendingX = 0
      this.pendingY = 0
      return(false)
    }
  }

  virtual JsmConnection[]? findConnToSelect(Int x,Int y)
  {
    //echo("Finding connection to select for container $name")
    JsmConnection[] insideConn := JsmConnection[,]
    children.each |r|
    {
      insideConn.addAll(r->findConnToSelect(x,y))
    }
    return(insideConn)
  }


  
  Void drawConnections(Graphics g)
  {
    children.each
    {
      //echo("Region.draw child $it.name")
      it.drawConnections(g)
    }
  }

  Int getLowestNode()
  {
    Int lowestY:=y1
    children.each 
    {  
      if ( it.y2 > lowestY) 
      {
        lowestY=it.y2
      }
    }
    return(lowestY)
  }

  virtual Void move(Int deltaX, Int deltaY)
  {
    x1+=deltaX
    y1+=deltaY
    x2+=deltaX
    y2+=deltaY
  }
  
  Void pendingMove(Int x,Int y)
  {
    if ( x >= parent.x1 && x <= parent.x2 && y >= parent.y1 && y <= parent.y2 )
    {
      pendingX=x
      pendingY=y
    }
  }
  
   
  Void addState(JsmState state)
  {
      children.add(state)
      state.parent=this
      states.add(state)
  }
  
  Void addChild(JsmNode child)
  {
    if ( ! children.contains(child))
    {
      children.add(child)
      child.parent=this
      if ( child.type == NodeType.STATE )
      {
        states.add(child)
      }
    }
    else
    {
      echo("[warn] $name already includes child $child.name")
    }
  }
  
    
  Void removeChild(JsmNode child)
  {
    children.remove(child)
    if ( child.type == NodeType.STATE )
    {
      states.remove(child)
    }
  }
  
  Void validate()
  {
    this.children.each 
    {   
      if ( it.parent != this ) 
      {
        if ( it.parent == null )
        {
          echo("[error] Region $this.name($this) child ${it.name}($it) null parent ")
        }
        else
        {
          echo("[error] Region $this.name child ${it.name} has mismatching parent ${it.parent.name}")
        }
      }
      if ( ! this.containsNode(it) && ! this.isRootState )
      {
        echo("[error] Region $this.details - child not in body ${it.details}")
      }
    }
  }
  
  
  // check is coordinate is inside the rectangle
  virtual Bool containsNode(JsmNode n)
  {
    return(contains(n.x1, n.y1, n.x2, n.y2))
  }
  
    
  // check is coordinate is inside the rectangle
  virtual Bool contains(Int x_1, Int y_1, Int x_2, Int y_2)
  {
    Bool rc
    if ( this.x1 <= x_1 && this.x2 >= x_2 && this.y1 <= y_1 && this.y2 >= y_2 )
    {
      echo("Container $name Contains node $this.x1,$this.y1,$this.x2,$this.y2 $x_1,$y_1,$x_2,$y_2 ")
      rc=true
    }
    else
    {
      rc=false
    }
    return(rc);
  }
  
  
  Str details()
  {
    return("[${this.name} x1:${this.x1},y1:${this.y1},x2:${this.x2},y2:${this.y2}]") 
  }
  
//  Str nextStateName()
//  {
//    return(this.parent.nextStateName())
//  }
  
  JsmState newState(Int nodeId,Int x,Int y)
  {
    //Str? newname
    //echo("Adding state to region $name")
//    if ( this.parent.parent == null ) // root state
//    {
//      newname="s${states.size + 1}"
//    }
//    else if ( parentState.regions.size == 1 )
//    {
//      newname="${parentState.name}.${states.size + 1}"
//    }
//    else
//    {
//      newname="${name}.${states.size + 1}"
//    }
    //newname=nextStateName()
    newname:="s"+nodeId
    JsmState node:=JsmState.maker(nodeId,newname,x,y,JsmOptions.instance.stateWidth,JsmOptions.instance.stateHeight)
    node.boxColor=Color.black
    addChild(node)
    return(node)
  }
  
  
  JsmFinal addFinal(Int nodeId,Int x,Int y)
  {
    Str newname:= "Final"
    JsmFinal node:=JsmFinal.maker(nodeId,newname,x,y,JsmOptions.instance.finalWidth,JsmOptions.instance.finalWidth)
    node.boxColor=Color.black
    addChild(node)
    return(node)
  }
  
  JsmJoin addJoin(Int nodeId,Int x,Int y)
  {
    Str newname:= "Join"
    JsmJoin node:=JsmJoin.maker(nodeId,newname,x,y,JsmOptions.instance.joinWidth,JsmOptions.instance.joinHeight)
    node.boxColor=Color.black
    addChild(node)
    return(node)
  }
  JsmFork addFork(Int nodeId,Int x,Int y)
  {
    Str newname:= "Fork"
    JsmFork node:=JsmFork.maker(nodeId,newname,x,y,JsmOptions.instance.joinWidth,JsmOptions.instance.joinHeight)
    node.boxColor=Color.black
    addChild(node)
    return(node)
  }
  JsmChoice addChoice(Int nodeId,Int x,Int y)
  {
    Str newname:= "Choice"
    JsmChoice node:=JsmChoice.maker(nodeId,newname,x,y,JsmOptions.instance.choiceWidth,JsmOptions.instance.choiceHeight)
    node.boxColor=Color.black
    addChild(node)
    return(node)
  }
  JsmJunction addJunction(Int nodeId,Int x,Int y)
  {
    Str newname:= "Junction"
    JsmJunction node:=JsmJunction.maker(nodeId,newname,x,y,JsmOptions.instance.junctionWidth,JsmOptions.instance.junctionWidth)
    node.boxColor=Color.black
    addChild(node)
    return(node)
  }
  
  JsmInitial? addInitial(Int nodeId,Int x,Int y)
  {
    JsmInitial? node
    Bool alreadyHasOne:=false
    children.each 
    { 
      //echo(it.typeof.toStr)
      if ( it.typeof.toStr  == "JsmGui::JsmInitial" )
      {
        alreadyHasOne=true
      } 
    }
    if ( alreadyHasOne )
    {
      // message box alerting user that there can only be a single initial state      
      echo("$this.name already has an initial state")
    }
    else
    {
      //echo("Creating new initial state")
      Str newname:= "Initial_$nodeId"
      node=JsmInitial.maker(nodeId,newname,x,y,JsmOptions.instance.initialWidth,JsmOptions.instance.initialWidth)
      node.boxColor=Color.black
      addChild(node)
    }
    return(node)
  }
  
  JsmNode? findNodeToSelect(Int x,Int y)
  {
    JsmNode? insideNode := null
    insideNode=states.eachWhile |state|
    { 
      echo("Region Look in state $state.name")
      return(state.findNodeToSelect(x,y))
    }
    if ( insideNode == null )
    {
	    insideNode=children.eachWhile |child|
	    { 
	      if ( child.type != NodeType.STATE && child.inBody(x, y) )
	      {
          insideNode=child
	        echo("JsmRegion.findNodeToSelect($x,$y) IN Region $this.name($this.x1,$this.y1,$this.x2,$this.y2)  PSEUDOSTATE $child.name ($child.x1,$child.y1,$child.x2,$child.y2) ")
          return(child)
	      }
        else
        {
          return(null) // no need to search any longer
	      } 
	    }
    }
    return(insideNode)    
  }
  
}