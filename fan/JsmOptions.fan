using gfx
using fwt

const class JsmOptions
{
  const Int stubLen:=10
  const Int arrowHeight:=8
  const Int arrowWidth:=3
  const Int stateWidth:=40
  const Int stateHeight:=30
  const Int finalWidth:=20
  const Int choiceHeight:=30
  const Int choiceWidth:=15
  const Int junctionWidth:=15
  const Int initialWidth:=20
  const Int joinWidth:=15
  const Int joinHeight:=30
  const Int regionMargin:=10
  const Int stateMargin:=10
  const File backupPath
  const File projectPath
  const Int cornerSize:=6
  const Int pseudoCornerSize:=3
  const Int cornerRounding:=24
  const Int minStateW:=40
  const Int minStateH:=30
  const Color cornerColor:=Color.black
  const Color stateColor:=Color.fromStr("#FFFFCC")
  const Color color:=Color.fromStr("#FFFFFF")
  const static JsmOptions instance := make()
  private new make() 
  { 
    backupPath=Uri("file:///c:/jsm/backup/").toFile()
    projectPath=Uri("file:///c:/jsm/").toFile()
    //File d:=Uri("file:///${backupPath}/").toFile
    //echo("backupPath ${backupPath.osPath}")
    //echo("projectPath ${projectPath.osPath}")
    if ( ! backupPath.exists )
    {
      backupPath.create
    }    
    //d=Uri("file:///${projectPath}/").toFile
    if ( ! projectPath.exists )
    {
      projectPath.create
    }    
  }
}