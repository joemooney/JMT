
@Serializable
class JsmTransition : JsmConnection
{
//  Str? guard
//  Str? event
//  Str? action
  new make(|This| f) : super(f)
  {
    //echo("making a new connection")
    f(this)
  }
  
  //new make(Str name,JsmNode source,JsmNode target,Side sourceSide,Side targetSide)
  new maker(Str name,JsmNode source,JsmNode target,Str connId): super(name,source,target,connId)
  {
  }
}
