using gfx
using fwt

@Serializable
class JsmState : JsmSmNode
{

  Int rounding:=0
  Int rounding2:=0
  Str entryActivity:=""
  Str exitActivity:=""
  Str doActivity:=""
  JsmDiagramSettings? settings
  
  virtual JsmRegion[] regions:=JsmRegion[,] 

  new make(|This| f) : super(f)
  {
    f(this)
    //echo("making a new state $name")
  }
  
  override Void restoreParentage([Int:JsmNode] nodeIds,JsmRegion? newParent )
  {
    
    this.parent=newParent
    nodeIds[this.nodeId]=this // restoreParentage for this node and add back global node id lookup
    regions.each
    {
      it.restoreParentage(nodeIds,this)
    }
    if ( ! isRootState ) // this.name != "rootState"
    {
      if ( newParent == null )
      {
        echo("[error] Restore state parentage $name ($this) newParent is null for non root state")
      }
      else
      {
        echo("Restore state parentage $name ($this) to region $newParent.name")
      }
    }
    else
    {
    }
  }
  
  override Void restoreConnections([Int:JsmNode] nodeIds)
  {
    echo("Restoring $sourceConnections.size source connections for state $name")
    super.restoreConnections(nodeIds)
    regions.each
    {
      it.restoreConnections(nodeIds)
    }
  }
  
  new maker(Int nodeId,Str name,Int x,Int y,Int w,Int h) : super (NodeType.STATE,nodeId,name,x,y,w,h)
  {

    minWidth=40
    minHeight=30
    this.fillColor = JsmOptions.instance.stateColor
    if ( x == 0 && y == 0 && w == 0 && h == 0 )
    {
      // root state
      x1=0
      y1=0
      x2=0
      y2=0
    }
    else
    {
	    if ( w < 0 )
	    {
	      this.x2=-1
	    } else if ( w < JsmOptions.instance.minStateW  )
	    {
	      this.x2=x1+JsmOptions.instance.minStateW;
	    }
	    if ( w < 0 )
	    {
	      this.y2=-1
	    } else if ( h < JsmOptions.instance.minStateH  )
	    {
	      this.y2=y1+JsmOptions.instance.minStateH;
	    }
    }
  }
  
  Void setRounding()
  {
    rounding=JsmOptions.instance.cornerRounding
    if ( x2 - x1 < rounding*2 )
    {
       rounding=(x2 -x1)/3
    }
    if ( y2 - y1 < rounding*2 )
    {
       rounding=(y2 -y1)/3
    }
    rounding2=rounding + rounding
    //echo("Rounding=$rounding")
  }
  
  Void drawRects(Graphics g)
  {
    g.brush = fillBrush()
    g.fillRect(x1+rounding, y1+rounding, x2 - x1 - rounding2, y2 - y1 - rounding2)
    g.fillRect(x1+rounding, y1, x2 - x1 - rounding2, rounding2)
    g.fillRect(x1+rounding, y2-rounding, x2 - x1 - rounding2, rounding)
    g.fillRect(x1, y1+rounding, rounding, y2 - y1 - rounding2)
    g.fillRect(x2-rounding, y1+rounding, rounding, y2 - y1 - rounding2)
    g.brush = Color.black
    g.drawLine(x1, y1 + rounding, x1, y2 - rounding)
    g.drawLine(x2, y1 + rounding, x2, y2 - rounding)
    g.drawLine(x1+rounding, y1, x2 - rounding + 1, y1)
    g.drawLine(x1+rounding, y2, x2 - rounding, y2)
    
  }
  
  Color fillBrush()
  {
    if ( this.fillColor == null )
    {
      return(JsmOptions.instance.stateColor)
    }
    else
    {
      return(this.fillColor)
    }
    
  }
  
  Void drawArcs(Graphics g)
  {
    //echo("rounding=$rounding")
    g.brush = fillBrush()
    g.fillArc(x1 , y1, rounding2, rounding2, 90, 90)
    g.fillArc(x1 , y2 - rounding2, rounding2, rounding2, 180, 90)
    g.fillArc(x2 - rounding2 , y1, rounding2, rounding2, 0, 90)
    g.fillArc(x2 - rounding2 , y2 - rounding2, rounding2, rounding2, 270, 90)
    g.brush = Color.black
    g.drawArc(x1 , y1 , rounding2 + 1, rounding2 + 1, 90, 90)
    g.drawArc(x1 , y2 - rounding2, rounding2, rounding2, 180, 90)
    g.drawArc(x2 - rounding2 , y1, rounding2, rounding2, 0, 90)
    g.drawArc(x2 - rounding2 , y2 - rounding2, rounding2, rounding2, 270, 90)
    
  }
  
  override Void drawName(Graphics g)
  {
    g.font = Desktop.sysFont.toSize(10)
    tw := g.font.width(this.name)
    tx := x1+((x2 - x1 - tw)/2) // center name in box
    ty := y1+5 // Down 20 from top of rect
    g.brush = Color.gray

    g.brush = Color.black
    g.drawText(this.name, tx, ty)
  }
  
  override Void drawDetails(Graphics g)
  {
    g.brush = Color.black
    g.drawLine(x1, y1+20, x2,y1+20)
    g.font = Desktop.sysFont.toSize(10)
    tw := g.font.width(this.name)
    g.drawText(this.name, x1+5, y1+25)
  }
  
  override Void calcConnections()
  {
    if ( regions.size > 0 )
    {
      regions.each
      {
        it.calcConnections()
      }
    }
    calcSideConnections(leftSlots  ,y1,y2,Axis.Y)
    calcSideConnections(rightSlots ,y1,y2,Axis.Y)
    calcSideConnections(topSlots   ,x1,x2,Axis.X)
    calcSideConnections(bottomSlots,x1,x2,Axis.X)
  }

  override Void draw(Graphics g)
  {
    //g.pen = Pen { width=8; join = Pen.joinBevel }
    
    //echo("Drawing state $name")
    if ( parent != null )
    {
      setRounding()
      drawRects(g)
      drawArcs(g)
      drawName(g)
      //drawDetails(g)
      //echo("draw connections for $name")
      //drawConnections(g)
      drawCorners(g,JsmOptions.instance.cornerSize) // only if hasFocus
    }
    if ( regions.size > 0 )
    {
      regions.each
      {
        //echo("State.draw region $it.name")
        it.draw(g)
      }
    }
    
  }
  
  // check is coordinate is inside the rectangle
  override Bool contains(Int x_1, Int y_1, Int x_2, Int y_2)
  {
    

    echo("Checking for intersection in state $name")
    if ( super.contains(x_1,y_1,x_2,y_2)  )
    {
      //echo("corner contained")
      return true
    }
    else
    {
      return false
    }
  }
  
  // check is coordinate is inside the rectangle but not overlapping a region border
  override Bool containsNode(JsmNode n)
  {
    if (super.containsNode(n))
    {
      if ( regions.size < 2 )
      {
        return(true)
      }
      else
      {
        JsmRegion? overlappingRegion:= regions.eachWhile |r|
        {
          if ( r.intersectsRegion(n))
          {
            return(r)
          }
          else
          {
            return(null) // null will cause the loop to continue to the next region
          }
        }
        if ( overlappingRegion == null )
        {
          return(true)
        }
        else
        {
          return(false)
        }
      }
    }
    else
    {
      return(false)
    }
  }
  
  ** Check is coordinate is close to dashed region separator 
  virtual JsmRegion? regionSelected(Int ev_x,Int ev_y)
  {
    // are we selecting the dashed region separator in order to move it
    JsmRegion? selectedRegion:= regions.eachWhile |r|
    {
      if ( r != regions.first() && r.closeToRegion(ev_x,ev_y ) )
      {
        echo("Selecting close to $this.name region $r.name $r.x1, $r.y1, $r.x2, $r.y2, $ev_x, $ev_y")
        return(r)
      }
      else
      {
        echo("NOT Selecting close to $this.name region $r.name $r.x1, $r.y1, $r.x2, $r.y2, $ev_x, $ev_y")
        return(null) // null will cause the loop to continue to the next region
      }
    }
    return(selectedRegion)
  }
  
//  Str nextStateName()
//  {
//    if ( this.isRootState )
//    {
//      // root state first region
//      return("s"+this.settings.
//    }
//    else
//    {
//      return(this.parent.nextStateName()) 
//    }
//  }
  
  
  // check is coordinate is inside the rectangle
  virtual JsmRegion? containingRegion(JsmNode n)
  {
    Bool intersecting:=false
    JsmRegion containingRegion:= regions.eachWhile |r|
    {
      if ( r.x1 <= n.x1 && r.x2 >= n.x2 && r.y1 <= n.y1 && r.y2 >= n.y2 )
      {
        return(r)
      }
      else
      {
        return(null) // null will cause the loop to continue to the next region
      }
    }
    echo("$this.name has region $containingRegion.name containing node $n.name")
    return(containingRegion)
  }
  
  override Void drawConnections(Graphics g)
  {
    super.drawConnections(g)
    if ( regions.size > 0 )
    {
      regions.each
      {
        //echo("State.draw region $it.name")
        it.drawConnections(g)
      }
    }
    
  }
  
  override JsmNode? findNodeToSelect(Int x,Int y)
  {
    JsmNode? insideNode := null
    //echo("State.findNodeToSelect in $name $x,$y -- regions=$regions.size")
    if ( regions.size == 0 )
    {
      //echo("S01")
      if ( inBody(x,y) == true )
      {
        //echo("S02")
        insideNode=this
      } 
      else
      {
        //echo("State.findNodeToSelect - not in body of state $name")
      }
    }
    else
    {
      //echo("S03")
	    insideNode=regions.eachWhile |r|
	    { 
        return(r->findNodeToSelect(x,y))
	    }
      if ( insideNode == null && inBody(x,y) == true )
      {
        //echo("S04")
        insideNode=this
        //echo("State.findNodeToSelect in state $name ($x,$y) -- insideNode=$insideNode.name")
      } 
    }
    if ( insideNode == null )
    {
      //echo("State.findNodeToSelect Did not find node in $name")
    }
    return(insideNode)    
  }
  
  override JsmConnection[]? findConnToSelect(Int x,Int y)
  {
    //echo("Finding connection to select for $name")
    JsmConnection[] insideConn := JsmConnection[,]
    regions.each |r|
    { 
      insideConn.addAll(r->findConnToSelect(x,y))
    }
    connections.each |c|
    { 
      if ( c.source == this && c.insideBody(x,y) )
      {
        insideConn.add(c)
      }
    }
    if ( insideConn.size == 0 )
    {
      //echo("Node.findConnToSelect not found in Conn $name $x,$y")
    }
    return(insideConn)    
  }
  
  Bool isRootState()
  {
    if ( this.settings != null )
    {
      echo("settings is null")
      return(true)
    }
    else
    {
      return(false)
    }
  }
  
  // Return the region containing x,y
  // This is only called in the even thtat we want a region to be added
  JsmRegion? getRegion(Int x,Int y,Bool createRegionIfNone)
  {
    JsmRegion? region
    if ( ! createRegionIfNone && regions.size == 0 )
    {
//      echo("This state has no region -- and we are not creating one")
      region=null
    }
    else if ( regions.size == 0)
    {
//      echo("This state has no region")
      region=addRegion()
    }
    else if ( isRootState ) // name == "rootState" )
    {
//      echo("This is the root state - $this.name")
      region=regions.first
    }
    else if ( regions.size == 1)
    {
//      echo("This state has only one region")
      region=regions.first
    }
    else    
    {
//      regions.each |r|
//      {
//        echo("$r.name -- $r.y2") 
//      }
      region=regions.eachWhile |r|
      {
        if ( y < r.y2 ) 
        {
          //echo("yes in this region"+this.details)
          return(r)
        }
        else
        {
          //echo("not in this region"+this.details)
          return(null)
        }
      }
    }
//    if ( region != null )
//    {
//      echo("--- in region $region.name $x,$y $region.y2 $this.regions.size")
//    }
//    else
//    {
//      echo("--- not in region")
//    }
    return(region)
  }
  
  override JsmNode[] getAllChildren()
  {
    JsmNode[] descendents := children.dup
    //echo("State.getAllChildren: $this.name -- $children.size direct sub-nodes -- this should be zero")
    regions.each 
    {   
      descendents.addAll(it.getAllChildren)  
    }
    //echo("getAllChildren: $this.name -- ${descendents.size - children.size} indirect sub-nodes")
    return(descendents)
  }
  
  JsmNode[] getImmediateChildren()
  {
    JsmNode[] immediateChildren := [,]
    echo("State.getImmediateChildren: $this.name -- $children.size direct sub-nodes -- this should be zero")
    regions.each 
    {   
      immediateChildren.addAll(it.children)
    }
    //echo("getAllChildren: $this.name -- ${descendents.size - children.size} indirect sub-nodes")
    return(immediateChildren)
  }
  
  override Void removeChild(JsmNode child)
  {
    regions.each 
    {   
      it.removeChild(child)  
    }
    super.removeChild(child)
  }
  
    
  virtual JsmState[] getSubstates()
  {
    JsmState[] substates := JsmState[,]
    //echo("State.getSubstates: $this.name ")
    regions.each 
    {   
      substates.addAll(it.states)
    }
    //echo("getSubstates: $this.name -- $substates.size substate")
    return(substates)
  }
  
  virtual JsmState[] getAllSubstates()
  {
    JsmState[] substates := JsmState[,]
    echo("State.getSubstates: $this.name ")
    regions.each 
    {   
      substates.addAll(it.getAllSubstates)
    }
    echo("getSubstates: $this.name -- $substates.size substate")
    return(substates)
  }
  
  JsmState newState(Int nodeId,Int x,Int y)
  {
    //echo("Adding state to $name")
    return(getRegion(x,y,true).newState(nodeId,x,y))
  }
  
  JsmFinal addFinal(Int nodeId,Int x,Int y)
  {
    return(getRegion(x,y,true).addFinal(nodeId,x,y))
  }
  
  JsmJoin addJoin(Int nodeId,Int x,Int y)
  {
    return(getRegion(x,y,true).addJoin(nodeId,x,y))
  }
  
  JsmFork addFork(Int nodeId,Int x,Int y)
  {
    return(getRegion(x,y,true).addFork(nodeId,x,y))
  }
  
  JsmChoice addChoice(Int nodeId,Int x,Int y)
  {
    return(getRegion(x,y,true).addChoice(nodeId,x,y))
  }
  
  JsmJunction addJunction(Int nodeId,Int x,Int y)
  {
    return(getRegion(x,y,true).addJunction(nodeId,x,y))
  }
  
  JsmInitial? addInitial(Int nodeId,Int x,Int y)
  {
    echo("Adding initial $nodeId,$x,$y "+this.details)
    return(getRegion(x,y,true).addInitial(nodeId,x,y))
  }
  
  JsmRegion firstRegion()
  {
    if ( regions.size == 0 )
    {
      addRegion()
    }
    return(regions[0])
  }
  
//  // return the containing state for this state
//  JsmState? parentState()
//  {
//    if ( this.parent != null )
//    {
//      return(this.parent.parent) // the parent of the containing region
//    }
//    else
//    {
//      return(null)
//    }
//  }
  
//  // return the containing state for this state
//  override JsmRegion? parentNode()
//  {
//    return(this.parent) // the parent of the containing region
//  }
  
  Void validate()
  {
    if ( regions.size == 0 )
    {
      if ( this.children.size != 0 )
      {
        echo("[error] State $name has no regions and $children.size children")
      }
      
    }
    else if ( regions.size == 1 && this.parent != null )
    {
      JsmRegion region:=this.firstRegion
      if ( region.x1 != this.x1 || region.x2 != this.x2 || region.y1 != this.y1 || region.y2 != this.y2 )
      {
        echo("[error] State $name with one region coordinate mismatch: $region.x1,$region.y1,$region.x2,$region.y2  $this.x1,$this.y1,$this.x2,$this.y2")
      }
    }
    this.regions.each { it.validate() }
  }
  
//  virtual Void addChildx(JsmNode child)
//  {
//    if ( ! children.contains(child))
//    {
//      children.add(child)
//      child.parent=this
//    }
//  }
  
  
  JsmRegion? addRegion()
  {
    Int ypos
    if ( regions.size == 0 )
    {
      ypos=y1
      //ypos=(y2-y1)/2
    }
    else if ( regions.last.children.size == 0 )
    {
      echo("[warn] Please add substates first")  
      return(null)
    }
    else
    {
      
      // horizontal region
      ypos=regions.last.getLowestNode() + JsmOptions.instance.regionMargin
      
      if ( ypos > y2 - JsmOptions.instance.stateHeight - JsmOptions.instance.stateMargin )
      {
        echo("Please enlarge state - not enough room to add new region")  
        return(null)
      }
      
      // horizontal region
      // update lowest region to start of new region
      regions.last.x2=this.x2
      regions.last.y2=ypos  
      
      //ypos=(y2 - regions.last.getLowestNode())/2
    }
    JsmRegion r:=JsmRegion.maker(this,"${this.name}_${this.regions.size + 1}",this.x1,ypos,this.x2,this.y2)
    r.parentState=this
    regions.add(r) 
    return(r)
  }
  
  override Void move(Int deltaX, Int deltaY)
  {

    super.move(deltaX, deltaY)
    regions.each { it.move(deltaX, deltaY) }
  }
  
  override Void resize(Int x, Int y)
  {
    super.resize(x, y)
    //echo("State.resize $name")
    if ( regions.size > 0)
    {
      switch(currentCorner)
      {
        case Corner.NE: 
          regions.each { it.x1 = this.x1 }
          regions[0].y1 = this.y1
        case Corner.NW: 
          regions.each { it.x2 = this.x2 }
          regions[0].y1 = this.y1
        case Corner.SE: 
          regions.each { it.x1 = this.x1 }
          regions[-1].y2 = this.y2
        case Corner.SW: 
          regions.each { it.x2 = this.x2 }
          regions[-1].y2 = this.y2
      }
        
    }
  }

  
  virtual Str getFirstRegionCoords()
  {
    Str out:="no_regions"
    if ( regions.size > 0) 
    {
      JsmRegion region:=this.firstRegion
      out="$region.x1,$region.y1,$region.x2,$region.y2"
    }
    return(out)    
  }
  
//  virtual Str details()
//  {
//    return("${this.name}")
//  }
  
  // check is coordinate is inside the rectangle
  JsmRegion? findRegionContainingNode(JsmNode n)
  {
    JsmRegion? region
    if ( regions.size == 0 )
    {
      if ( this.contains(n.x1, n.y1, n.x2, n.y2) )
      {
        echo("Added node to regionless state $name")
        region=this.addRegion() // side effect of adding region
      }
      else
      {
        echo("Not in this region of state $name")
      }
    }
    else
    {
      region=regions.eachWhile |  r|
      {
        echo ("$r.name $r.x1,$r.y1,$r.x2,$r.y2 -- $n.x1,$n.y1,$n.x2,$n.y2  == $this.x1,$this.y1,$this.x2,$this.y2")
        if(r.contains(n.x1, n.y1, n.x2, n.y2))
        {
          echo("Found node $n.name in region $r.name of $name")
          return(r)
        }
        else
        {
          return(null)
        }
      }
    }
    return(region)
  }
  
  ** Return the region before this region
  JsmRegion? previousRegion(JsmRegion r2)
  {
    JsmRegion? prevRegion:=null
    regions.eachWhile |r|
    {
      if ( r == r2)
      {
        return("break")
      }
      else
      {
        prevRegion=r
        return(null)
      }
    }
    return(prevRegion)
  }  
}
