using gfx
using fwt

@Serializable
class JsmDiagramSettings
{
  Int stubLen:=10
  Int arrowHeight:=8
  Int arrowWidth:=3
  Int stateWidth:=40
  Int stateHeight:=30
  Int finalWidth:=20
  Int choiceHeight:=30
  Int choiceWidth:=15
  Int junctionWidth:=15
  Int initialWidth:=20
  Int joinWidth:=15
  Int joinHeight:=30
  Int regionMargin:=10
  Int stateMargin:=10
  Str? backupPath
  Str  codeIndent:="   "
  Str? projectPath
  Int cornerSize:=6
  Int pseudoCornerSize:=3
  Str newLine:="\n"
  Int cornerRounding:=24
  Int minStateW:=40
  Int minStateH:=30
  Color cornerColor:=Color.black
  Color stateColor:=Color.fromStr("#FFFFCC")
  Color color:=Color.fromStr("#FFFFFF")
  Str diagramName:="sm1"
  Str? diagramPath
  
  new make() 
  { 
//    diagramPath=projectPath.osPath
//    //echo("backupPath ${backupPath.osPath}")
//    //echo("projectPath ${projectPath.osPath}")
//    if ( ! backupPath.exists )
//    {
//      backupPath.create
//    }    
//    if ( ! projectPath.exists )
//    {
//      projectPath.create
//    }    
  }
  
  File diagramDirObj()
  {
    echo("----")
    echo(diagramPath)
    path:=""
    if (this.diagramPath.indexr("\\",-1) == null )
    {
      path="c:/"
    }
    else
    {
      path=this.diagramPath[0..this.diagramPath.indexr("\\",-1)].replace("\\", "/")
    }
    echo("Getting dir object for $path")
    return(JsmUtil.getFileObj1(path))
  }
  
  Str diagramFile()
  {
    path:=""
    if (this.diagramPath.indexr("\\",-1) == null )
    {
      path="unknown.txt"
    }
    else
    {
      path=this.diagramPath[this.diagramPath.indexr("\\",-1)+1..-1]
    }
    echo("diagram file $path")
    return(path)
  }
}
