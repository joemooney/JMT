using gfx
using fwt

@Serializable
class JsmCircleNode : JsmNode
{
  Int rounding:=0
  Int rounding2:=0
  
  new make(|This| f) : super(f)
  {
    //echo("making a new circle node")
    f(this)
  }
  
  new maker(NodeType type,Int nodeId,Str name,Int x,Int y,Int w,Int h) : super (type,nodeId,name,x,y,w,h)
  {
    minWidth=20
    minHeight=20
  }
  
  override Void resize(Int x, Int y)
  {
    super.resize(x, y)
    makeSquare()
  }
  
  override Void draw(Graphics g)
  {
    g.brush = Color.black
    g.fillOval(x1+1, y1+1, x2-x1-2, y2-y1-2)
    drawConnections(g)
    drawCorners(g,JsmOptions.instance.pseudoCornerSize)
    //drawPendingConnection(g)
  }
  
  
  
}
