using gfx
using fwt

@Serializable
class JsmNode
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
  Str? spec
  //Int w
  //Int h
  //Str name
  //Int x1
  //Int y1
  //Int x2
  //Int y2
  //Int pendingX
  //Int pendingY
  //Int minWidth:=20
  //Int minHeight:=20
  @Transient virtual JsmRegion? parent
  @Transient Str errorMsg:=""
  virtual NodeType type
  //@Transient virtual JsmNode? parent
  //virtual JsmNode[] children:=JsmNode[,] 

  virtual JsmConnection[] sourceConnections
  @Transient virtual JsmConnection[]? connections:=JsmConnection[,] 
  @Transient virtual JsmConnection[]? leftSlots:=JsmConnection[,] 
  @Transient virtual JsmConnection[]? rightSlots:=JsmConnection[,] 
  @Transient virtual JsmConnection[]? topSlots:=JsmConnection[,] 
  @Transient virtual JsmConnection[]? bottomSlots:=JsmConnection[,] 
  virtual JsmNode[] children


  Color boxColor:= Color.black
  @Transient Bool hasFocus:=false
  Corner currentCorner := Corner.NOT_CORNER
  
  //Color boxColor:= Color.black
  Color? fillColor
  //@Transient Bool hasFocus:=false
  Int nodeId
  //Corner currentCorner := Corner.NOT_CORNER
  
  new make(|This| f)
  {
    f(this)
    //echo("making a new node $name")
  }
  
  Bool isDefaultSize()
  {
    if (this.width == this.minWidth && this.height == this.minHeight ) 
    {
      return(true)  
    }
    else
    {
      return(false)
    }
  }
  
  Int width()
  {
    return(this.x2 - this.x1) 
  }
    
  Int height()
  {
    return(this.y2 - this.y1) 
  }
    
  // return the containing state for this state
  virtual JsmNode? parentNode()
  {
    if ( this.parent != null)
    {
      return(this.parent.parent) 
    }
    else
    {
      return(null) // return the node that contains this node
    }
  }
  
  new maker(NodeType type,Int nodeId,Str name,Int x,Int y,Int w,Int h)
  {
    children=JsmNode[,] 
    this.nodeId=nodeId
    
    this.sourceConnections=JsmConnection[,] 
    this.name = name;
    this.type=type;
    this.x1=x;
    this.y1=y;
    //this.w=w;
    //this.h=h;
    this.x2=x1+w;
    this.y2=y1+h;
  }

  Str details()
  {
    return("[${this.name} x1:${this.x1},y1:${this.y1},x2:${this.x2},y2:${this.y2}]") 
  }
  
  virtual Void restoreParentage([Int:JsmNode] nodeIds,JsmRegion? newParent )
  {
    echo("Restoring parentage for $name")
    this.parent=newParent
    nodeIds[this.nodeId]=this // restore parentage for node and add to global lookup table 
  }
  
  virtual Void restoreConnections([Int:JsmNode] nodeIds)
  {
    echo("Restoring $sourceConnections.size source connections for $name")
    this.sourceConnections.each 
    {   
      if ( nodeIds.containsKey(it.sourceNodeId))
      {
        it.source = nodeIds[it.sourceNodeId]
      }
      else
      {
        echo("Error: Missing node $it.sourceNodeId") 
      }
      if ( nodeIds.containsKey(it.targetNodeId))
      {
        it.target = nodeIds[it.targetNodeId]
      }
      else
      {
        echo("Error: Missing node $it.targetNodeId") 
      }
      it.target.restoreConnection(it,it.targetSide)
      it.source.restoreConnection(it,it.sourceSide)
    }
  }
  
  virtual Void restoreConnection(JsmConnection conn,Side side)
  {
    connections.add(conn)
    switch(side)
    {
      case Side.TOP:    topSlots.add(conn)
      case Side.BOTTOM: bottomSlots.add(conn)
      case Side.LEFT:   leftSlots.add(conn)
      case Side.RIGHT:  rightSlots.add(conn)
      default: 
    }
  }
  
  virtual Void drawName(Graphics g)
  {
  }
  
  virtual Void drawDetails(Graphics g)
  {
  }
  
  virtual Void drawConnections(Graphics g)
  {
    if ( parent != null )
    {
      drawPendingConnection(g)
    }
    this.drawSideConnections(g,topSlots)
    this.drawSideConnections(g,bottomSlots)
    this.drawSideConnections(g,leftSlots)
    this.drawSideConnections(g,rightSlots)
  }
  
  virtual Void drawSideConnections(Graphics g,JsmConnection[] slots)
  {
    g.brush = Color.black
    //echo("draw $connections.size connections")
    slots.each |conn|  // are all ordered at this stage
    { 
      if ( conn.source == this)
      {
        //echo("Draw $conn.source.name -> $conn.target.name connection in $this.name")
        conn.draw(g)
      }
    }
  }
  
  Void remove(JsmNode[] deletedNodes)
  {
    deletedNodes.each 
    {   
      if ( it != this )
      {
        this.removeNode(it)
      }
    }
  }
  
  virtual JsmNode[] getAllChildren()
  {
    JsmNode[] descendents := children.dup
    //echo("Node.getAllChildren: $this.name -- $children.size direct sub-nodes")
    children.each 
    {   
      descendents.addAll(it.getAllChildren)  
    }
    //echo("getAllChildren: $this.name -- ${descendents.size - children.size} indirect sub-nodes")
    return(descendents)
  }
  
  virtual Void removeChild(JsmNode child)
  {
    children.remove(child)
  }
  
  Void addChild(JsmNode child)
  {
    if ( ! children.contains(child))
    {
      children.add(child)
    }
  }
  
  virtual Void removeConn(JsmConnection conn)
  {
    this.topSlots.remove(conn)
    this.bottomSlots.remove(conn)
    this.leftSlots.remove(conn)
    this.rightSlots.remove(conn)
    this.connections.remove(conn)
    this.sourceConnections.remove(conn)
  }
  
  
  virtual Void removeNode(JsmNode deletedNode)
  {
    if ( deletedNode == this)
    {
      return;
    }
    connections=this.connections.exclude |conn|
    {   
      filter:=false
      if ( conn.source == deletedNode || conn.target == deletedNode )
      {
        this.topSlots.remove(conn)
        this.bottomSlots.remove(conn)
        this.leftSlots.remove(conn)
        this.rightSlots.remove(conn)
        filter=true
        //echo("Removing $conn.source.name -> $conn.target.name connection from $this.name")
      }
      else
      {
        //echo("Not Removing $conn.source.name -> $conn.target.name connection from $this.name")
      }
      return(filter)
    }
    //echo("connections=$connections.size")
  }
  
  // we only have connections where we are the source 
  virtual Void makeConnection(JsmConnection conn)
  {
      if ( conn.source.y2 + (JsmOptions.instance.stubLen*2) <= conn.target.y1 )
      {
        conn.sourceSide=Side.BOTTOM
        conn.targetSide=Side.TOP
      }
      else if ( conn.source.y1 >= conn.target.y2 + (JsmOptions.instance.stubLen*2) )
      {
        conn.sourceSide=Side.TOP
        conn.targetSide=Side.BOTTOM
      }
      else if ( conn.source.x2  < conn.target.x1 )
      {
        conn.sourceSide=Side.RIGHT
        conn.targetSide=Side.LEFT
      }
      else 
      {
        conn.sourceSide=Side.LEFT
        conn.targetSide=Side.RIGHT
      }
      conn.source.connectToSide(conn.sourceSide, conn)
      conn.target.connectToSide(conn.targetSide, conn)
  }
  

  
  // When we move a node we need to check if the connections need to change sides
  virtual Void checkSwitchSides()
  {
    connections.each |conn|
    {
      //echo("x: $conn.sourceSide.toStr() -> $conn.targetSide.toStr()")
      Side oldSourceSide:=conn.sourceSide
      conn.updateSourceSide()
      if ( conn.sourceSide != oldSourceSide )      
      {
        //echo("Update Source Side")
        conn.source.removeSideConnection(oldSourceSide,conn)
        conn.source.connectToSide(conn.sourceSide, conn)
      }
      else
      {
        conn.source.orderSlots(conn.sourceSide)
      }
      //echo("y: $conn.sourceSide.toStr() -> $conn.targetSide.toStr()")
      Side oldTargetSide:=conn.targetSide
      conn.updateTargetSide()
      if ( conn.targetSide != oldTargetSide )      
      {
        //echo("Update Target Side")
        conn.target.removeSideConnection(oldTargetSide,conn)
        conn.target.connectToSide(conn.targetSide, conn)
      }
      else
      {
        conn.target.orderSlots(conn.targetSide)
      }
      //echo("z: $conn.sourceSide.toStr() -> $conn.targetSide.toStr()")
    }
  }
  
  virtual Void removeSideConnection(Side side,JsmConnection conn)
  {
    switch(side)
    {
      case Side.TOP:    topSlots.remove(conn)
      case Side.BOTTOM: bottomSlots.remove(conn)
      case Side.LEFT:   leftSlots.remove(conn)
      case Side.RIGHT:  rightSlots.remove(conn)
      default: 
    }
  }
  
  virtual Void reorderSlots()
  {
      sortSlots(Axis.X,topSlots)
      sortSlots(Axis.X,bottomSlots)
      sortSlots(Axis.Y,leftSlots)
      sortSlots(Axis.Y,rightSlots)
  }
  
  virtual Void orderSlots(Side side)
  {
    //echo("Sort $name slots for $side")
    switch(side)
    {
      case Side.TOP:    sortSlots(Axis.X,topSlots)
      case Side.BOTTOM: sortSlots(Axis.X,bottomSlots)
      case Side.LEFT:   sortSlots(Axis.Y,leftSlots)
      case Side.RIGHT:  sortSlots(Axis.Y,rightSlots)
      default: 
    }
  }
  
  virtual Bool sameConnNodes(JsmConnection a,JsmConnection b)
  {
    if ( a.source == b.source && a.target == b.target 
    ||   a.source == b.target && a.target == b.source 
      )
    {
      echo("Same nodes $a.source.name,$a.target.name $b.source.name,$b.target.name")
      return(true)
    }
    else
    {
      return(false)
    }
  }
  
  virtual Void sortSlots(Axis axis,JsmConnection[] slots)
  {
    //echo("Sort slots $slots.size for $axis")
    slots.sort |JsmConnection a, JsmConnection b->Int| 
    { 
      Int rc:=0
      if ( sameConnNodes(a,b) == true )
      {
        if ( a.connId < b.connId )
        {
          echo("Same nodes ${a.connId} < ${b.connId} ")
          rc=-1
        }
        else
        {
          echo("Same nodes ${a.connId} > ${b.connId} ")
          rc=1
        }
      }
      else if ( a.source == this && b.source == this)
      {
        if ( axis == Axis.X && a.target.middleX < b.target.middleX 
        ||   axis == Axis.Y && a.target.middleY < b.target.middleY ) 
        {
          rc=(-1); 
        }
        else
        {
          rc=(+1)
        }
      }
      else if ( a.source == this && b.target == this)
      {
        if ( axis == Axis.X && a.target.middleX < b.source.middleX 
        ||   axis == Axis.Y && a.target.middleY < b.source.middleY ) 
        {
          rc=-1
        }
        else
        {
          rc=+1
        }
      }
      else if ( a.target == this && b.target == this)
      {
        if ( axis == Axis.X && a.source.middleX < b.source.middleX 
        ||   axis == Axis.Y && a.source.middleY < b.source.middleY ) 
        {
          rc=-1
        }
        else
        {
          rc=+1
        }
      }
      else // if ( a.target == this && b.source == this)
      {
        if ( axis == Axis.X && a.source.middleX < b.target.middleX 
        ||   axis == Axis.Y && a.source.middleY < b.target.middleY ) 
        {
          rc=-1
        }
        else
        {
          rc=+1
        }
      }
      if ( rc < 0 )
      {
          echo("$axis $a.source.name,$a.target.name < $b.source.name,$b.target.name ")
      }
      else
      {
          echo("$axis $a.source.name,$a.target.name > $b.source.name,$b.target.name ")
      }
      return(rc)
    }
  }
  
  virtual Void connectToSide(Side s, JsmConnection c)
  {
    switch(s)
    {
      case Side.TOP:    connectToSlot(Axis.X,topSlots,c)
      case Side.BOTTOM: connectToSlot(Axis.X,bottomSlots,c)
      case Side.LEFT:   connectToSlot(Axis.Y,leftSlots,c)
      case Side.RIGHT:  connectToSlot(Axis.Y,rightSlots,c)
      default: 
    }
      
  }
  
  virtual Void connectToSlot(Axis axis,JsmConnection[] slots, JsmConnection newConn)
  {
    Int insertPoint:=0
    slots.eachWhile |conn|
    {   
      //echo("$newConn.target.middleY < $conn.target.middleY")
      if ( axis == Axis.X && newConn.target.middleX < conn.target.middleX 
      ||   axis == Axis.Y && newConn.target.middleY < conn.target.middleY ) 
      {
        return(true); 
      }
      else
      {
        insertPoint++
        return(false)
      }
    }
    //echo("Insertion point $insertPoint ($slots.size) in $this.name  ($newConn.source.name -> $newConn.target.name)")
    //if ( slots.size < insertPoint)
    //{
    //  echo("add connection")
    //  slots.add(newConn) 
    //}
    //else
    //{
      //echo("insert connection $this.name")
      slots.insert(insertPoint,newConn) 
    //}
    //echo("--------------- $slots.toStr")
    //slots.each { echo("   ($it.source.name -> $it.target.name)") }
    //updateSlotPositions(axis,slots)
  }
  
  // Establish Connection:
  // figure out which sides should be connected
  // add a new slot to both sides
  // find the insertion point in the slots in both sides
  //
  // Establish connection coordinates for both ends of connections
  //   for each node establish its coordinates for each connection
  //   regardless of whether it is the source or target
  //
  // Draw Connection:
  
  virtual Void calcConnections()
  {
    calcSideConnections(leftSlots  ,y1,y2,Axis.Y)
    calcSideConnections(rightSlots ,y1,y2,Axis.Y)
    calcSideConnections(topSlots   ,x1,x2,Axis.X)
    calcSideConnections(bottomSlots,x1,x2,Axis.X)
  }
  
  
  virtual Void calcSideConnections(JsmConnection[] slots,Int p1,Int p2,Axis axis)
  {
    Int pos:=0;
    Int inc:=(p2-p1)/(slots.size+1)
    
    // distribution may be even, 
    slots.each |conn|
    {   
      pos+=inc;
      if ( conn.source == this )
      {
        if ( axis ==   Axis.X )
        {
           conn.sourceX=pos;
           conn.sourceY=0;
        }
        else
        {
           conn.sourceX=0;
           conn.sourceY=pos;
        }
        //echo("calc ($conn.source.name $conn.sourceX,$conn.sourceY -> $conn.target.name)")
      }
      else
      {
        if ( axis ==   Axis.X )
        {
           conn.targetX=pos;
           conn.targetY=0;
        }
        else
        {
           conn.targetX=0;
           conn.targetY=pos;
        }
        //echo("Calc ($conn.source.name -> $conn.target.name $conn.targetX,$conn.targetY )")
      }
    }
  }
  
  
  virtual Void drawCorners(Graphics g, Int cornerSize)
  {
    if (hasFocus)
    {
      g.brush = JsmOptions.instance.cornerColor
      g.fillRect(x1, y1, cornerSize, cornerSize)    // top left
      g.fillRect(x2-cornerSize-1, y1, cornerSize, cornerSize)  // top right
      g.fillRect(x1, y2-cornerSize-1, cornerSize, cornerSize)    // bottom left
      g.fillRect(x2-cornerSize-1, y2-cornerSize-1, cornerSize, cornerSize)    // bottom right
    }
  }
  
  virtual Void drawPendingConnection(Graphics g)
  {
      if ( pendingX != 0 )
    {
       g.brush = Color.blue
       //echo("g.drawLine($name, ${middleX()},${middleY()}, ${pendingX}, ${pendingY}")    // top left
       g.drawLine(middleX(),middleY(), pendingX, pendingY)    // top left
    }
  }
  
  virtual Void draw(Graphics g)
  {
  }
  
  Int middleX()
  {
    return(x1+(x2 - x1)/2)
  }
  
  Int middleY()
  {
    return(y1+(y2 - y1)/2)
  }
  
//  Bool checkFocus(Int x, Int y)
//  {
//    if ( x >= x1 && x <= x2 && y >= y1 && y <= y2 )
//    {
//      hasFocus=true;
//    }
//    else
//    {
//      hasFocus=false;
//    }
//    return(hasFocus);
//  }
  
  virtual Void resize(Int x, Int y)
  {
    //echo("Node.resize $name")
    // save old coords
    Int oldX1:=x1
    Int oldX2:=x2
    Int oldY1:=y1
    Int oldY2:=y2
    if ( currentCorner == Corner.NE )
    {
      x1+=x
      y1+=y
    }
    else if ( currentCorner == Corner.SW )
    {
      x2+=x
      y2+=y
    }
    else if ( currentCorner == Corner.NW )
    {
      x2+=x
      y1+=y
    }
    else if ( currentCorner == Corner.SE )
    {
      x1+=x
      y2+=y
    }
    else
    {
    }
    // restore the coord if the new coords are less than the minimums
    if ( x2 - x1 < minWidth )
    {
      x1=oldX1
      x2=oldX2
    }
    if ( y2 - y1 < minHeight )
    {
      y1=oldY1
      y2=oldY2
    }
  }
  
    
  Void makeSquare()
  {
    if ( x2 - x1 > y2 - y1) // wider than longer make same
    {
       x2=x2 - ((x2 - x1) - (y2 - y1))
      
    }
    else if ( y2 - y1 > x2 - x1) // longer than wide make same
    {
       y2=y2 - ((y2 - y1) - (x2 - x1))
    }
  }
  
  virtual Str coords()
  {
    return("$x1,$y1,$x2,$y2")
  }
  
  virtual Void move(Int deltaX, Int deltaY)
  {
    x1+=deltaX
    y1+=deltaY
    x2+=deltaX
    y2+=deltaY
  }
  
  Int getCornerY(Corner c)
  {
    switch(c) 
    {
      case Corner.SW: return(y2)
      case Corner.SE: return(y2)
      case Corner.NW: return(y1)
      case Corner.NE: return(y1)
      default: return(0)
    }
  }
  
  Int getCornerX(Corner c)
  {
    switch(c) 
    {
      case Corner.SW: return(x2)
      case Corner.NW: return(x2)
      case Corner.SE: return(x1)
      case Corner.NE: return(x1)
      default: return(0)
    }
  }
  
  Int getCurrentCornerX()
  {
    return(getCornerX(this.currentCorner))
  }
  
  Int getCurrentCornerY()
  {
    return(getCornerY(this.currentCorner))
  }
  
  Corner getCorner(Int x, Int y)
  {
    if ( x >= x1 && x <= x1 + 5 && y >= y1 && y <= y1 + 5 )
    {
      currentCorner=Corner.NE
    }
    else if ( x >= x1 && x <= x1 + 5 && y <= y2 && y >= y2 - 5 )
    {
      currentCorner=Corner.SE
    }
    else if ( x <= x2 && x >= x2 - 5 && y >= y1 && y <= y1 + 5 )
    {
      currentCorner=Corner.NW
    }
    else if ( x <= x2 && x >= x2 - 5 && y <= y2 && y >= y2 - 5 )
    {
      currentCorner=Corner.SW
    }
    else
    {
      currentCorner=Corner.NOT_CORNER
    }
    return(currentCorner);
  }
  
  Void pendingConnection(Int x,Int y)
  {
    pendingX=x
    pendingY=y
  }
  
  Void startConnection()
  {
    pendingX=middleX()
    pendingY=middleY()
  }
  
  JsmConnection? endConnection(JsmNode target)
  {
    JsmConnection? newConn
    if ( validTarget(target) == true )
    {
      Str newname:= "c_"+this.name+"_"+(connections.size + 1)
      newConn = JsmConnection.maker(newname,this,target,"${this.name}_${target.name}_${connections.size+1}")
      this.sourceConnections.add(newConn)
      this.connections.add(newConn)
      target.connections.add(newConn)
      makeConnection(newConn)
    }
    else
    {
      echo("Invalid target $target.name $target.typeof.toStr ")
    }
    return(newConn)
  }
  
  Void reportError(Str msg)
  {
    if ( this.parentNode == null )
    {
      this.errorMsg=msg
    }
    else
    {
      this.parentNode.reportError(msg)
    }
  }
  
  virtual JsmConnection[]? findConnToSelect(Int x,Int y)
  {
    //echo("Finding connection to select for $name")
    JsmConnection[] insideConn := JsmConnection[,]
    children.each |r|
    { 
      insideConn.addAll(r->findConnToSelect(x,y))
    }
    echo("Checking connections for $this.name")
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
  
  virtual JsmNode? findNodeToSelect(Int x,Int y)
  {
    JsmNode? insideNode := null
    insideNode=children.eachWhile |r|
    { 
      return(r->findNodeToSelect(x,y))
    }
    if ( insideNode == null && inBody(x,y) == true )
    {
      insideNode=this
      //echo("Node.findNodeToSelect found in node $name $x,$y")
    }
    if ( insideNode == null )
    {
      //echo("Node.findNodeToSelect not found in node $name $x,$y")
    }
    return(insideNode)    
  }
  
  virtual Bool validTarget(JsmNode target)
  {
    if ( target.typeof.toStr  == "JsmGui::JsmInitial" )
    {
      echo("Invalid Target $target.name $target.typeof.toStr ")
        return false
    }     
    else
    {
      //echo("Valid target $target.name $target.typeof.toStr ")
      return true
    }
  }
  
  // check is coordinate is inside the rectangle
  Bool inBody(Int x, Int y)
  {
    Bool rc
    
    if ( this.x1 == 0 && this.x2 == 0 && this.y1 == 0 && this.y2 == 0 )
    {
      rc=true
    }
    else if ( x >= x1 && x <= x2 && y >= y1 && y <= y2 )
    {
      //echo("Node.inBody(true) $x,$y [$this.x1,$this.y1,$this.x2,$this.y2] ")
      rc=true
    }
    else
    {
      //echo("not inBody $name $x,$y [$this.x1,$this.y1,$this.x2,$this.y2] ")
      rc=false
    }
    return(rc);
  }
  
  // check if this node is smaller than other node
  Bool isSmallerThan(JsmNode otherNode)
  {
    if ( (this.x2 - this.x1) < (otherNode.x2 - otherNode.x1) )
    {
      return(true)
    }
    else
    {
      return(false)
    }
  }
  
  // check is coordinate is inside the rectangle
  virtual Bool contains(Int x_1, Int y_1, Int x_2, Int y_2)
  {
    Bool rc
    if ( this.x1 <= x_1 && this.x2 >= x_2 && this.y1 <= y_1 && this.y2 >= y_2 )
    {
      //echo("Node $name Contains node $this.x1,$this.y1,$this.x2,$this.y2 $x_1,$y_1,$x_2,$y_2 ")
      rc=true
    }
    else
    {
      rc=false
    }
    return(rc);
  }
  
  // check is coordinate is inside the rectangle
  virtual Bool containsNode(JsmNode n)
  {
    return(contains(n.x1, n.y1, n.x2, n.y2))
  }
  
  // check is coordinate is inside the rectangle
  ** Given a node check if this node overlaps either partially or fully
  ** returning true if one contains the other or they intersect
  virtual Bool overlapsNode(JsmNode n)
  {

    echo("Node Checking for intersection in $name of $n.name")
    if ( inBody(n.x1, n.y1) || inBody(n.x2, n.y1) || inBody(n.x1, n.y2) || inBody(n.x2, n.y2) 
    ||   n.inBody(this.x1, this.y1) 
    ||   n.inBody(this.x2, this.y1) 
    ||   n.inBody(this.x1, this.y2) 
    ||   n.inBody(this.x2, this.y2) 
    ||   ( n.x1 <= this.x1 && n.x2 >= this.x1 && n.y1 >= this.y1 && n.y1 <= this.y2 )
    ||   ( n.x1 >= this.x1 && n.x1 <= this.x2 && n.y1 <= this.y1 && n.y2 >= this.y1 )
       )
    {
      //echo("corner contained")
      return true
    }
    else
    {
      return false
    }
  }
  
  // check is coordinate is partially inside the rectangle
  ** Check if this line segment intersects this node
  Bool intersects(Int x1, Int y1, Int x2, Int y2)
  {
    Bool rc
    if ( contains(x1,x2,x2,y2) )
    {
      //echo("Contained within node")
      rc=false 
    }
    else if (( this.x1 >= x1 && this.x1 <= x2 ) 
          || ( this.x2 >= x1 && this.x2 <= x2 ) 
          || ( this.y1 <= y1 && this.y1 <= y2 )
          || ( this.y2 <= y1 && this.y2 <= y2 ))
    {
      rc=true
    }
    else
    {
      //echo("Not intersecting nodes $this.x1,$this.y1,$this.x2,$this.y2 $x1,$y1,$x2,$y2 ")
      rc=false
    }
    return(rc);
  }
  
  
  // check is coordinate is inside the rectangle
  Bool inArea(Int areaX1, Int areaY1, Int areaX2, Int areaY2)
  {
    Bool rc
    if ( areaX1 <= x1 && areaX2 >= x2 && areaY1 <= y1 && areaY2 >= y2 )
    {
      rc=true
    }
    else
    {
      rc=false
    }
    return(rc);
  }
  
}
