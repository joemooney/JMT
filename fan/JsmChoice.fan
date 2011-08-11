using gfx
using fwt

@Serializable
class JsmChoice : JsmNode
{
  Int rounding:=0
  Int rounding2:=0
  
  new make(|This| f) : super(f)
  {
    //echo("making a new choice")
    f(this)
  }
  
  new maker(Int nodeId,Str name,Int x,Int y,Int w,Int h) : super (NodeType.CHOICE,nodeId,name,x,y,w,h)
  {
    minWidth=15
    minHeight=30
  }
  
  
  override Void draw(Graphics g)
  {
    g.brush = Color.black
    g.drawLine(this.middleX, y1, x1, this.middleY)
    g.drawLine(this.middleX, y1, x2, this.middleY)
    g.drawLine(this.middleX, y2, x1, this.middleY)
    g.drawLine(this.middleX, y2, x2, this.middleY)
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
       //echo("g.drawLine($name, ${middleX()},${middleY()}, ${pendingX}, ${pendingY}")    // top left
       g.drawLine(middleX(),middleY(), pendingX, pendingY)    // top left
    }
  }
  
}
