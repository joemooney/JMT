using gfx
using fwt

@Serializable
class JsmInitial : JsmPseudoState
{
  
  new make(|This| f) : super(f)
  {
    //echo("making a new initial")
    f(this)
  }
  
  new maker(Int nodeId,Str name,Int x,Int y,Int w,Int h) : super (NodeType.INITIAL,nodeId,name,x,y,w,h)
  {
    minWidth=20
    minHeight=20
    this.fillColor=Color.black
  }
  
  override Void draw(Graphics g)
  {
    g.brush = this.fillColor
    g.fillOval(x1+1, y1+1, x2-x1-2, y2-y1-2)
    drawConnections(g)
    drawCorners(g,JsmOptions.instance.pseudoCornerSize)
    //drawPendingConnection(g)
  }
    
  override Void resize(Int x, Int y)
  {
    super.resize(x, y)
    makeSquare()
  }
  
  override Bool validTarget(JsmNode target)
  {
    return(true)
//    if ( target.typeof.toStr  != "JsmGui::JsmState" )
//    {
//      echo("Initial Invalid Target $target.name $target.typeof.toStr ")
//        return false
//    }     
//    else
//    {
//      echo("Initial valid Target $target.name $target.typeof.toStr ")
//      return true
//    }
  }
  
}
