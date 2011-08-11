
@Serializable
class JsmPseudoState : JsmSmNode
{
  
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
  

}
