
@Serializable
class JsmLineSegment
{
  Int x1
  Int y1
  Int x2
  Int y2
   
  Int real_x1
  Int real_y1
  Int real_x2
  Int real_y2
   
  
  new make(|This| f)
  {
    //echo("making a new state")
    f(this)
  }
  
  //new make(Str name,JsmNode source,JsmNode target,Side sourceSide,Side targetSide)
  new maker(Int x1,Int y1,Int x2, Int y2)
  {
    this.x1=x1;
    this.y1=y1;
    this.x2=x2;
    this.y2=y2;
  }
  
  virtual JsmLineSegment? closeToLine(Int x0,Int y0)
  {
    Int proximity := 5
    if (( (real_x1 < real_x2) && ((x0 < (real_x1 - proximity )) || (x0 > (real_x2 + proximity ) ) ) ) 
    ||  ( (real_x1 > real_x2) && ((x0 < (real_x2 - proximity )) || (x0 > (real_x1 + proximity ) ) ) )
    ||  ( (real_y1 < real_y2) && ((y0 < (real_y1 - proximity )) || (y0 > (real_y2 + proximity ) ) ) ) 
    ||  ( (real_y1 > real_y2) && ((y0 < (real_y2 - proximity )) || (y0 > (real_y1 + proximity ) ) ) ) )
    {
      //echo("Not proximate to line segment is $real_x1,$real_y1,$real_x2,$real_y2 - $x0,$y0 ")
        return(null)       
    }
    
    Float d1:= (real_x2 - real_x1) * (real_y1 - y0).toFloat;
    Float d2:= (real_x1 - x0) * (real_y2 - real_y1).toFloat;
    Float d3:= d1 - d2;
    Float d4:= d3.abs;
    Float d5:= (real_x2 - real_x1) * (real_x2 - real_x1).toFloat;
    Float d6:= (real_y2 - real_y1) * (real_y2 - real_y1).toFloat;
    Float d7:= d5+d6;
    Float d8:= d7.sqrt;
    Float d:= d4 / d8;
    if ( d7 == 0.0f )
    {
      d=0.0f
    }
    
    // s / ( ( (x2-x1)*(x2-x1) ) - ( (y2-y1)*(y2-y1) ) ).sqrt
    // Float d:=Float.abs( (x2-x1)*(y1-y0) - (x1-x0)(y2-y1) ) / Float.sqrt( ( (x2-x1)*(x2-x1) ) - ( (y2-y1)*(y2-y1) ) )
    //echo("Distance to line segment is $real_x1,$real_y1,$real_x2,$real_y2 - $x0,$y0 - $d6 $d4/$d7 <<<$d>>>")
    if( d.toInt < proximity)
    {
      return(this)
    }
    else
    {
      return(null)
    }
  }
  
}
