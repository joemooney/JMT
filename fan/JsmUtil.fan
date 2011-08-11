
class JsmUtil
{
  static Bool closeToLine(Int x1,Int y1,Int x2,Int y2,Int x0,Int y0)
  {
    Float d1:= ((x2 - x1) * (y1 - y0)).toFloat;
    Float d2:= ((x1 - x0) * (y2 - y1)).toFloat;
    Float d3:= d1 - d2;
    Float d4:= d3.abs;
    Float d5:= ((x2 - x1) * (x2 - x1)).toFloat;
    Float d6:= ((y2 - y1) * (y2 - y1)).toFloat;
    Float d7:= (d5 - d6).sqrt;
    Float d:= d4 / d7;
    
    // s / ( ( (x2-x1)*(x2-x1) ) - ( (y2-y1)*(y2-y1) ) ).sqrt
    // Float d:=Float.abs( (x2-x1)*(y1-y0) - (x1-x0)(y2-y1) ) / Float.sqrt( ( (x2-x1)*(x2-x1) ) - ( (y2-y1)*(y2-y1) ) )
    echo("Distance to is $x1,$y1,$x2,$y2 - $x0,$y0 - $d1,$d2,$d3,$d4,$d5,$d6,$d6,$d")
    //Float dx := JsmUtil.FindDistanceToSegment(x1,y1,x2,y2,x0,y0)
    //echo("Distance to is $x1,$y1,$x2,$y2 - $x0,$y0 - $d1,$d2,$d3,$d4,$d5,$d6,$d6,$d --- $dx")
    if( d < 5.1f)
    {
      //echo("-> $d  $x1,$y1,$x2,$y2,$x0,$y0******************************")
      return(true)
    }
    else
    {
      //echo("$d  $x1,$y1,$x2,$y2,$x0,$y0******************************")
      return(false)
    }
  }
  
  static File getFileObj2(File dir,Str file)
  {
    return(Uri("file:///"+dir.osPath.replace("\\", "/")+"/"+file).toFile)
  }
  
  static File getFileObj1(Str file)
  {
    echo("hi")
    return(Uri("file:///"+file).toFile)
  }
  
//  static File getDirObj1(Str file)
//  {
//    return(Uri("file:///"+file).toDir)
//  }
  
  // From: http://blog.csharphelper.com/2010/03/26/find-the-shortest-distance-between-a-point-and-a-line-segment-in-c.aspx
  
  // Calculate the distance between
  // point pt and the segment p1 --> p2.
  static Float FindDistanceToSegment(Int x1,Int y1,Int x2,Int y2,Int x0,Int y0)
  {
    Float dx := (x2 - x1).toFloat;
    Float dy := (y2 - y1).toFloat;
    if ((dx == 0.0f) && (dy == 0.0f))
    {
        // It's a point not a line segment.
        dx = (x0 - x1).toFloat;
        dy = (y0 - y1).toFloat;
        return (dx * dx + dy * dy).sqrt;
    }

    // Calculate the t that minimizes the distance.
    Float t := ((x0 - x1) * dx + (y0 - y1) * dy) / (dx * dx + dy * dy);

    // See if this represents one of the segment's
    // end points or a point in the middle.
    if (t < 0.0f)
    {
        dx = (x0 - x1).toFloat;
        dy = (y0 - y1).toFloat;
    }
    else if (t > 1.0f)
    {
        dx = (x0 - x2).toFloat;
        dy = (y0 - y2).toFloat;
    }
    else
    {
        dx = x0 - x1 + t * dx
        dy = y0 - y1 + t * dy
    }
    return ((dx * dx + dy * dy).sqrt);
  }
  
  // Calculate the distance between
  // point pt and the segment p1 --> p2.
  static Float[] FindClosestPointOnSegment(Int x1,Int y1,Int x2,Int y2,Int x0,Int y0)
  {
    Float dx := (x2 - x1).toFloat;
    Float dy := (y2 - y1).toFloat;
    if ((dx == 0.0f) && (dy == 0.0f))
    {
        // It's a point not a line segment.
        return ([x1.toFloat,y1.toFloat])
    }

    // Calculate the t that minimizes the distance.
    Float t := ((x0 - x1) * dx + (y0 - y1) * dy) / (dx * dx + dy * dy);

    // See if this represents one of the segment's
    // end points or a point in the middle.
    if (t < 0.0f)
    {
        dx = x1.toFloat;
        dy = y1.toFloat;
    }
    else if (t > 1.0f)
    {
        dx = x2.toFloat;
        dy = y2.toFloat;
    }
    else
    {
        dx = x1 + t * dx
        dy = y1 + t * dy
    }

    return([dx,dy])
  }
}
