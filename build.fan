using build
class Build : build::BuildPod
{
  new make()
  {
    podName = "JsmGui"
    summary = ""
    srcDirs = [`fan/`, `fan/images/`]
    depends = ["sys 1.0","gfx 1.0","fwt 1.0"]
  }
}
