
@Serializable
class JsmSmNode : JsmNode
{
  new make(|This| f) : super(f)
  {
    //echo("making a new circle node")
    f(this)
  }
  
  new maker(NodeType type,Int nodeId,Str name,Int x,Int y,Int w,Int h) : super (type,nodeId,name,x,y,w,h)
  {
    
  }
  
  // return the containing state for this state
  JsmState? parentState()
  {
    if ( this.parent != null )
    {
      return(this.parent.parent) // the parent of the containing region
    }
    else
    {
      return(null)
    }
  }
}
