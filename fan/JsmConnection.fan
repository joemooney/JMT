using gfx
using fwt

enum class ConnStyle { LINE, ARC, U_SHAPE, L_SHAPE, STEP, S_SHAPE }

@Serializable
class JsmConnection
{
  Str? name
  @Transient virtual JsmNode? source
  @Transient virtual JsmNode? target
  Int? sourceNodeId
  Int? targetNodeId
  JsmLineSegment[]? lineSegments
  Side? sourceSide
  Side? targetSide
  Int? sourceX
  Int? sourceY
  Int? targetX
  Int? targetY
  Str connId
  Str event:="none"
  Str guard:="none"
  Str action:="none"
  Bool? internalTx:=false
  ConnStyle style
  @Transient Bool selected:=false
  
  new make(|This| f)
  {
    //echo("making a new connection")
    f(this)
  }
  
  //new make(Str name,JsmNode source,JsmNode target,Side sourceSide,Side targetSide)
  new maker(Str name,JsmNode source,JsmNode target,Str connId)
  {
    this.lineSegments=JsmLineSegment[,]
    this.name = name;
    this.source=source;
    this.target=target;
    this.sourceNodeId = source.nodeId
    this.targetNodeId = target.nodeId
    this.connId=connId;
    this.style=ConnStyle.LINE;
    
    setInitialSegments();
    //this.sourceSide=sourceSide;
    //this.targetSide=targetSide;
  }
  
  Void setInitialSegments()
  {
      this.lineSegments.add(JsmLineSegment.maker(-1,-1,-2,-2));     // source(-1) to source stub(-2)
      this.lineSegments.add(JsmLineSegment.maker(-2,-2,-3,-3));   // source stub(-2) to target stub(-3)
      this.lineSegments.add(JsmLineSegment.maker(-3,-3,-4,-4));     //  target stub(-3) to target(-4)
  }
  
  virtual Void drawName(Graphics g)
  {
  }
  
  virtual Void remove()
  {
    this.source.removeConn(this)
    this.target.removeConn(this)
  }
  
  virtual Void drawConnection(Graphics g)
  {
  }
  virtual Bool insideBody(Int x,Int y)
  {
    Bool rc:=false
    switch(this.style)
    {
      case ConnStyle.LINE:
        segment:=lineSegments.eachWhile |segment|
        {
          return(segment.closeToLine(x,y))
        }
        if ( segment != null )
        {
          //echo("$this.name rc set to true")
          //echo("$this.name close to line == true")
          rc=true
        }
        else
        {
          //echo("$this.name close to line == false")
        }
      default:
        echo("Invalid line style for $name")
    }
    //echo("$this.name rc=$rc")
    return(rc) 
  }
  
  virtual Bool closeToLine(Int x1,Int y1,Int x2,Int y2,Int x0,Int y0)
  {
    Float d1:= (x2 - x1) * (y1 - y0).toFloat;
    Float d2:= (x1 - x0) * (y2 - y1).toFloat;
    Float d3:= d1 - d2;
    Float d4:= d3.abs;
    Float d5:= (x2 - x1) * (x2 - x1).toFloat;
    Float d6:= (y2 - y1) * (y2 - y1).toFloat;
    Float d7:= d6.sqrt;
    Float d:= d4 / d7;
    
    // s / ( ( (x2-x1)*(x2-x1) ) - ( (y2-y1)*(y2-y1) ) ).sqrt
    // Float d:=Float.abs( (x2-x1)*(y1-y0) - (x1-x0)(y2-y1) ) / Float.sqrt( ( (x2-x1)*(x2-x1) ) - ( (y2-y1)*(y2-y1) ) )
    //echo("Distance to $name is $x1,$y1,$x2,$y2 - $x0,$y0 - $d4, $d")
    if( d < 5.1f)
    {
      echo("**********************************")
      return(true)
    }
    else
    {
      return(false)
    }
  }
  
  virtual Int originX()
  {
    if ( sourceSide == Side.RIGHT )
    {
      return(source.x2+sourceX);
    }
    else
    {
      return(source.x1+sourceX);
    }
  }
  
  virtual Int originY()
  {
    if ( sourceSide == Side.BOTTOM )
    {
      return(source.y2+sourceY);
    }
    else
    {
      return(source.y1+sourceY);
    }
  }
  
  virtual Int destX()
  {
    if ( targetSide == Side.RIGHT )
    {
      return(target.x2+targetX);
    }
    else
    {
      return(target.x1+targetX);
    }
  }
  
  virtual Int destY()
  {
    if ( targetSide == Side.BOTTOM )
    {
      return(target.y2+targetY);
    }
    else
    {
      return(target.y1+targetY);
    }
  }
  
  Int getXcoord(Int c)
  {
    Int coord:=c
    if ( c == -1 )
    {
      coord=originX
    }
    else if ( c == -4 )
    {
      coord=destX
    }
    else if ( c == -2 )
    {
      switch(sourceSide) 
      {
        case Side.TOP:
        case Side.BOTTOM: 
          coord=originX;
        case Side.RIGHT:  
          coord=originX+JsmOptions.instance.stubLen;
        case Side.LEFT:   
          coord=originX-JsmOptions.instance.stubLen;
        default: 
      }
    }
    else if ( c == -3 )
    {
      switch(targetSide) 
      {
        case Side.TOP:
        case Side.BOTTOM: 
          coord=destX;
        case Side.RIGHT:  
          coord=destX+JsmOptions.instance.stubLen;
        case Side.LEFT:   
          coord=destX-JsmOptions.instance.stubLen;
        default: 
      }
    }
    return(coord)
  }
  
  Int getYcoord(Int c)
  {
    Int coord:=c
    if ( c == -1 )
    {
      coord=originY
    }
    else if ( c == -4 )
    {
      coord=this.destY
    }
    else if ( c == -2 )
    {
      switch(sourceSide) 
      {
        case Side.RIGHT:  
        case Side.LEFT:   
          coord=originY;
        case Side.TOP:
          coord=originY-JsmOptions.instance.stubLen;
        case Side.BOTTOM: 
          coord=originY+JsmOptions.instance.stubLen;
        default: 
      }
    }
    else if ( c == -3 )
    {
      switch(targetSide) 
      {
        case Side.RIGHT:  
        case Side.LEFT:  
          coord=destY;
        case Side.TOP:
          coord=destY-JsmOptions.instance.stubLen;
        case Side.BOTTOM: 
          coord=destY+JsmOptions.instance.stubLen;
        default: 
      }
    }
    return(coord)
  }
  
  
  virtual Void draw(Graphics g)
  {
    if ( this.selected )
    {
      //echo("conn selected")
      g.brush=Color.orange;
    }
    else
    {
      //echo("conn not selected")
      g.brush=Color.black;
    }
    Int _x1:=0;
    Int _y1:=0;
    Int _x2:=0;
    Int _y2:=0;
    
    lineSegments.each 
    {  
      _x1=getXcoord(it.x1);
      it.real_x1=_x1;
      _y1=getYcoord(it.y1);
      it.real_y1=_y1;
      _x2=getXcoord(it.x2);
      it.real_x2=_x2;
      _y2=getYcoord(it.y2);
      it.real_y2=_y2;
      g.drawLine(_x1,_y1,_x2,_y2);
      //echo("--$_x1,$_y1,$_x1,$_y1,$it.x2,$it.y2")
      if ( it.x2 == -4 ) // endSegment draw arrow
      {
        drawEnd(g,_x2,_y2)
      }
    }
    
    //xdraw(g)
  }
  
  
  virtual Void drawMiddle(Graphics g,Int x,Int y)
  {
       Int x1:=target.x1+targetX;
       Int y1:=target.y1+targetY;
       Int x2:=target.x2+targetX;
       Int y2:=target.y2+targetY;
        switch(targetSide) 
        {
          case Side.TOP:
                //echo("target is top")
                g.drawLine(x,y,x1,y1-JsmOptions.instance.stubLen);
                g.drawLine(x1,y1-JsmOptions.instance.stubLen,x1,y1);
                g.drawLine(x1-JsmOptions.instance.arrowWidth,y1-JsmOptions.instance.arrowHeight,x1,y1);
                g.drawLine(x1+JsmOptions.instance.arrowWidth,y1-JsmOptions.instance.arrowHeight,x1,y1);
          case Side.BOTTOM: 
                echo("target is bottom: $x,$y,$x1,$y2 + $JsmOptions.instance.stubLen");
                g.drawLine(x,y,x1,y2+JsmOptions.instance.stubLen);
                echo("target is bottom: $x,$y2 - $JsmOptions.instance.stubLen,$x1,$y2");
                g.drawLine(x1,y2+JsmOptions.instance.stubLen,x1,y2);
                g.drawLine(x1-JsmOptions.instance.arrowWidth,y2+JsmOptions.instance.arrowHeight,x1,y2);
                g.drawLine(x1+JsmOptions.instance.arrowWidth,y2+JsmOptions.instance.arrowHeight,x1,y2);
          case Side.RIGHT:  
                g.drawLine(x,y,x2+JsmOptions.instance.stubLen,y1);
                g.drawLine(x2+JsmOptions.instance.stubLen,y1,x2,y1);
                g.drawLine(x2+JsmOptions.instance.arrowHeight,y1-JsmOptions.instance.arrowWidth,x2,y1);
                g.drawLine(x2+JsmOptions.instance.arrowHeight,y1+JsmOptions.instance.arrowWidth,x2,y1);
          case Side.LEFT:   
                g.drawLine(x,y,x1-JsmOptions.instance.stubLen,y1);
                g.drawLine(x1-JsmOptions.instance.stubLen,y1,x1,y1);
                g.drawLine(x1-JsmOptions.instance.arrowHeight,y1-JsmOptions.instance.arrowWidth,x1,y1);
                g.drawLine(x1-JsmOptions.instance.arrowHeight,y1+JsmOptions.instance.arrowWidth,x1,y1);
          default: 
        }
        g.brush=Color.red;
        g.drawLine(originX(),originY(),destX(),destY())
  }
  
  
  virtual Void drawEnd(Graphics g,Int x,Int y)
  {
        switch(targetSide) 
        {
          case Side.TOP:
                //echo("target is top")
                g.drawLine(x-JsmOptions.instance.arrowWidth,y-JsmOptions.instance.arrowHeight,x,y);
                g.drawLine(x+JsmOptions.instance.arrowWidth,y-JsmOptions.instance.arrowHeight,x,y);
          case Side.BOTTOM: 
                g.drawLine(x-JsmOptions.instance.arrowWidth,y+JsmOptions.instance.arrowHeight,x,y);
                g.drawLine(x+JsmOptions.instance.arrowWidth,y+JsmOptions.instance.arrowHeight,x,y);
          case Side.RIGHT:  
                g.drawLine(x+JsmOptions.instance.arrowHeight,y-JsmOptions.instance.arrowWidth,x,y);
                g.drawLine(x+JsmOptions.instance.arrowHeight,y+JsmOptions.instance.arrowWidth,x,y);
          case Side.LEFT:   
                g.drawLine(x-JsmOptions.instance.arrowHeight,y-JsmOptions.instance.arrowWidth,x,y);
                g.drawLine(x-JsmOptions.instance.arrowHeight,y+JsmOptions.instance.arrowWidth,x,y);
          default: 
        }
  }
  
  virtual Bool updateSourceSide()
  {
    Side side:=sourceSide
    if ( source.y2 + (JsmOptions.instance.stubLen*2) <= target.y1 )
    {
        sourceSide=Side.BOTTOM
    }
    else if ( source.y1 >= target.y2 + (JsmOptions.instance.stubLen*2) )
    {
        sourceSide=Side.TOP
    }
    else if ( source.x2  < target.x1 )
    {
        sourceSide=Side.RIGHT
    }
    else 
    {
        sourceSide=Side.LEFT
    }
    if  ( sourceSide == side )
    {
      return(false);
    }
    else
    {
      return(true);
    }
  }
  
  // calculate the side of the target to connect to
  virtual Bool updateTargetSide()
  {
    Side side:=targetSide
    if ( source.y2 + (JsmOptions.instance.stubLen*2) <= target.y1 )
    {
        targetSide=Side.TOP
    }
    else if ( source.y1 >= target.y2 + (JsmOptions.instance.stubLen*2) )
    {
        targetSide=Side.BOTTOM
    }
    else if ( source.x2  < target.x1 )
    {
        targetSide=Side.LEFT
    }
    else 
    {
        targetSide=Side.RIGHT
    }
    if  ( targetSide == side )
    {
      return(false);
    }
    else
    {
      return(true);
    }
  }
  
}
