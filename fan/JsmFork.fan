using gfx
using fwt

@Serializable
class JsmFork : JsmNode
{
  Int rounding:=0
  Int rounding2:=0
  
  new make(|This| f) : super(f)
  {
    //echo("making a new fork")
    f(this)
  }
  
  new maker(Int nodeId,Str name,Int x,Int y,Int w,Int h) : super (NodeType.FORK,nodeId,name,x,y,w,h)
  {
    minWidth=15
    minHeight=30
  }
  
  
  override Void draw(Graphics g)
  {
    g.brush = Color.black
    g.fillRect(x1+5, y1, x2-x1-10, y2-y1)
    drawConnections(g)
    if (hasFocus)
    {
       g.brush = JsmOptions.instance.cornerColor
       g.fillRect(x1, y1, JsmOptions.instance.pseudoCornerSize, JsmOptions.instance.pseudoCornerSize)    // top left
       g.fillRect(x2-JsmOptions.instance.pseudoCornerSize-1, y1, JsmOptions.instance.pseudoCornerSize, JsmOptions.instance.pseudoCornerSize)  // top right
       g.fillRect(x1, y2-JsmOptions.instance.pseudoCornerSize-1, JsmOptions.instance.pseudoCornerSize, JsmOptions.instance.pseudoCornerSize)    // bottom left
       g.fillRect(x2-JsmOptions.instance.pseudoCornerSize-1, y2-JsmOptions.instance.pseudoCornerSize-1, JsmOptions.instance.pseudoCornerSize, JsmOptions.instance.pseudoCornerSize)    // bottom right
    }
    if ( pendingX != 0 )
    {
       g.brush = Color.black
       echo("g.drawLine($name, ${middleX()},${middleY()}, ${pendingX}, ${pendingY}")    // top left
       g.drawLine(middleX(),middleY(), pendingX, pendingY)    // top left
    }
  }
  
}
