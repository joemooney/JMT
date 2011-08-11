using gfx
using fwt

@Serializable
class JsmJunction : JsmCircleNode
{
  
  new make(|This| f) : super(f)
  {
    //echo("making a new junction")
    f(this)
  }
  
  
  new maker(Int nodeId,Str name,Int x,Int y,Int w,Int h) : super (NodeType.JUNCTION,nodeId,name,x,y,w,h)
  {
    minWidth=15
    minHeight=15
    this.fillColor = Color.blue
  }
  
    
  override Void resize(Int x, Int y)
  {
    super.resize(x, y)
    makeSquare()
  }
  
 
  override Void draw(Graphics g)
  {
    g.brush = this.fillColor
    g.fillOval(x1+1, y1+1, x2-x1-2, y2-y1-2)
    drawConnections(g)
    drawCorners(g,JsmOptions.instance.pseudoCornerSize)
    //drawPendingConnection(g)
  }
  
}
