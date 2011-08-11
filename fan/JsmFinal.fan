using gfx
using fwt

@Serializable
class JsmFinal : JsmNode
{
  Int rounding:=0
  Int rounding2:=0
  
  new make(|This| f) : super(f)
  {
    //echo("making a new final")
    f(this)
  }
  
  new maker(Int nodeId,Str name,Int x,Int y,Int w,Int h) : super (NodeType.FINAL,nodeId,name,x,y,w,h)
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
    g.fillOval(x1+4, y1+4, x2-x1-7, y2-y1-7)
    g.drawOval(x1+1, y1+1, x2-x1-2, y2-y1-2)
    drawConnections(g)
    drawCorners(g,JsmOptions.instance.pseudoCornerSize)
    //drawPendingConnection(g)
//    if (hasFocus)
//    {
//       g.brush = JsmOptions.instance.cornerColor
//       g.fillRect(x1, y1, JsmOptions.instance.pseudoCornerSize, JsmOptions.instance.pseudoCornerSize)    // top left
//       g.fillRect(x2-JsmOptions.instance.pseudoCornerSize-1, y1, JsmOptions.instance.pseudoCornerSize, JsmOptions.instance.pseudoCornerSize)  // top right
//       g.fillRect(x1, y2-JsmOptions.instance.pseudoCornerSize-1, JsmOptions.instance.pseudoCornerSize, JsmOptions.instance.pseudoCornerSize)    // bottom left
//       g.fillRect(x2-JsmOptions.instance.pseudoCornerSize-1, y2-JsmOptions.instance.pseudoCornerSize-1, JsmOptions.instance.pseudoCornerSize, JsmOptions.instance.pseudoCornerSize)    // bottom right
//    }
//    if ( pendingX != 0 )
//    {
//       g.brush = Color.black
//       echo("g.drawLine($name, ${middleX()},${middleY()}, ${pendingX}, ${pendingY}")    // top left
//       g.drawLine(middleX(),middleY(), pendingX, pendingY)    // top left
//    }
  }
  
}
