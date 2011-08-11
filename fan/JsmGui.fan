//
// Copyright (c) 2008, Brian Frank and Andy Frank
// Licensed under the Academic Free License version 3.0
//
// History:
//   10 Jun 08  Brian Frank  Creation
//

using gfx
using fwt

enum class NodeType { STATE, JOIN, FORK, JUNCTION, INITIAL, FINAL, CHOICE }
enum class Side { NONE, TOP, BOTTOM, LEFT, RIGHT }
enum class Axis { X, Y }
enum class Corner { NE, NW, SE, SW, NOT_CORNER }
enum class EditMode { ARROW, SELECT, MODE_MOVE, RESIZE, 
                      ADD_STATE, ENTER_CONNECT, CONNECT, ADD_FINAL, ADD_INITIAL, 
                      ADD_JOIN, ADD_FORK, ADD_CHOICE, ADD_JUNCTION,
                      ADD_CLASS, MOVE_REGION }
enum class AlignMode { CENTER, MIDDLE, LEFT, RIGHT, TOP, BOTTOM }
**
** JsmGui displays the FWT sampler program.
** 
class JsmGui
{
  AlignMode alignMode:=AlignMode.CENTER
  EditMode EditMode:=EditMode.ARROW
  Label? statusBar
  Window? mainWindow
  TabPane? tabs
  JsmDiagram? currentDiagram
  Int:JsmDiagram diagrams := Int:JsmDiagram[:]  // Hash Map
  EventRegistry? eventRegistry

  **
  ** Put the whole thing together in a tabbed pane
  **
  Void main()
  {
    tabs = TabPane
    {
//  Tab { text = "State Diagram";  InsetPane { makeStateDiagram, }, },
          //Tab { text = "State Diagram";  InsetPane { makeStateDiagram, }, },
//        Tab { text = "Buttons";        InsetPane { makeButtons, }, },
//        Tab { text = "Labels";         InsetPane { makeLabels, }, },
//        Tab { text = "ProgessBar";     InsetPane { makeProgressBar, }, },
          Tab { text = "HelpBrowser";     InsetPane { makeWebBrowser, }, },
//        Tab { text = "Text";           InsetPane { makeText, }, },
//        Tab { text = "BorderPane";     InsetPane { makeBorderPane, }, },
//        Tab { text = "EdgePane";       InsetPane { makeEdgePane, }, },
//        Tab { text = "GridPane";       InsetPane { makeGridPane, }, },
          Tab { text = "Explorer";       InsetPane { makeTreeAndTable, }, },
//        Tab { text = "Window";         InsetPane { makeWindow, }, },
//        Tab { text = "Serialization";  InsetPane { makeSerialization, }, },
//        Tab { text = "Graphics";       InsetPane { makeGraphics, }, },
    }
    openStateDiagram(null,"sm_1",null)
    tabs.onSelect.add |Event ev| { selectNewTab(ev)   }
    
    mainWindow=Window
    {
      title = "JMT - Joe's Modeling Toolkit"
      size = Size(1000, 800)
      menuBar = makeMenuBar
      content = EdgePane
      {
        top = makeToolBar
        center = tabs
        bottom = makeStatusBar
      }
    }.open
  }
  
  Void warnUser(Str msg)
  {
     Dialog.openWarn(this.mainWindow, msg)
  }
  
  Void selectNewTab(Event ev)
  {
    this.currentDiagram=diagrams.get(ev.index)
    if ( currentDiagram != null  )
    {
      echo("changed to diagram $this.currentDiagram.settings.diagramName ($ev.index)")
    }
    else
    {
      echo("Selected a non-diagram tab")
    }
  }
  
  // if this is a new diagram then we check to ensure that the path does not
  // already exist.
  // With no name supplied we prompt for a diagram name.
  // With no path supplied we default to the project directory.
  // We never call this with a null name but with a path
  //
  // If we are opening an existing path we load that state object first and 
  // then call this function
  // We then restore the settings in Diagram.restoreState which includes the
  // name and the path
  JsmDiagram? openStateDiagram(Bool? isNew, Str? newDiagramName,Str? newDiagramPath)
  {
    JsmDiagram? newDiagram
    
    if ( newDiagramName == null )
    {
      newDiagramName=Dialog.openPromptStr(this.mainWindow, "New State Diagram Name:")
    }
    if ( newDiagramName != null )
    {
      if ( ! alreadyOpen(newDiagramName))
      {
	      if ( newDiagramPath == null )
	      {
	        newDiagramPath=JsmUtil.getFileObj2(JsmOptions.instance.projectPath, newDiagramName+".txt").osPath
	      }
        File f:= JsmUtil.getFileObj2(JsmOptions.instance.projectPath, newDiagramName+".txt")
        if ( isNew == true  && f.exists )
        {
          Dialog.openErr(this.mainWindow, "State Diagram $newDiagramName already exists")
        }
        else
        {
          openEventRegistry()
		      newDiagram=JsmDiagram(this,newDiagramName,newDiagramPath)
          // this will be overwritten if this is an existing diagram
	        newDiagram.stateMachineCanvas.rootState.settings=newDiagram.settings
		      Tab newTab:=Tab { text = "State Diagram";  InsetPane { newDiagram.mainPane, }, }
		      tabs.tabs.index(newTab)
		      newDiagram.diagramTab=newTab
		      diagrams.add(tabs.tabs.size, newDiagram)
		      newTab.text=newDiagramName
		      newTab.image=this.stateIcon
		      this.tabs.add(newTab)
		      this.tabs.selected=newTab
		      this.tabs.relayout
	        this.currentDiagram=newDiagram
        }
      }
    }
    return(newDiagram)
  }
  
  Void openEventRegistry()
  {
	  if ( eventRegistry == null )
	  {
      Obj? obj:=null
	    File f2:=JsmUtil.getFileObj2(JsmOptions.instance.projectPath,"events.txt")
	    if ( f2.exists )
	    {
        try
        {
	        obj=f2.readObj
	        if ( obj.typeof.toStr == "JsmGui::EventRegistry" )
	        {
	          echo("loading project event registry")
	          // this is the object we just loaded
	          eventRegistry=obj
            echo("[info] loaded $eventRegistry.lookup.size events")
	        }
        }
        catch (Err e)
        {
          echo("Failed to load event registry: $e.toStr")
        }
  	    if ( obj == null || obj.typeof.toStr != "JsmGui::EventRegistry" )
        {
	        obj=null
	        echo("[error] Invalid event registry on file")
        }
	    }
	    if ( obj == null )
      {
	      this.eventRegistry=EventRegistry.maker()
	      echo("[info] Created new event registry on file")
	      echo(this.eventRegistry.events.size)
      }
      eventRegistry.file=f2
	  }
  }
  
  Bool alreadyOpen(Str newDiagramName)
  {
	  Bool? alreadyExists:=diagrams.eachWhile |diagram|
	  {   
	    if ( diagram.settings.diagramName == newDiagramName) 
	    {
	      return(true)
	    }
	    else
	    {
	      return(null)
	    }
	  }
	  if (  alreadyExists != null )
	  {
	    warnUser("A diagram named ${newDiagramName} is already open")
      return(true)   
	  }
    else
    {
      return(false)   
    }
  }
  
  Void closeAction(Event e)
  {
    Bool asked:=false
    Str proceed:="Yes"
    if ( this.currentDiagram != null)
    {
      if ( diagrams[this.tabs.index(this.tabs.selected)] == this.currentDiagram)
      {
        echo("Close diagram ${currentDiagram.settings.diagramName}") 
        if ( this.currentDiagram.notSaved() )
        {
          proceed=Dialog.openInfo(e.window, "There are unsaved changes - discard?", Dialog.yesNo).toStr
          asked=true
        }
        if ( proceed == "Yes" )
        {
          this.diagrams.remove(this.tabs.selectedIndex)
          this.tabs.remove(this.tabs.selected) 
        }
      }
      else
      {
        echo("Do not Close diagram ${currentDiagram.settings.diagramName}") 
      }
    }
    if ( proceed == "Yes" && this.eventRegistry.changed )
    {
      if ( asked == false )
      {
        proceed=Dialog.openInfo(e.window, "There are unsaved changes to event registry - discard?", Dialog.yesNo).toStr
      }
      if ( proceed == "Yes" )
      {
      }
    }
  }
  
  Void openAction(Event e)
  {
    File? f:=FileDialog { dir=JsmOptions.instance.projectPath }.open(e.window)
    if ( f != null )
    {
	    Obj o:=f.readObj
	    if ( o.typeof.toStr == "JsmGui::JsmState" )
	    {
	      echo("yes this si a state")
	      // this is the object we just loaded
	      JsmState s:=o
	      if ( ! alreadyOpen(s.settings.diagramName) )
	      {
          // this is an existing state diagram
          // provide name and path 
	        newDiagram:=openStateDiagram(false,s.settings.diagramName,s.settings.diagramPath)     
          // set the root state to the object we read from the file
          newDiagram.restoreState(s)
	      }
	    }
	    else
	    {
	      echo("no this is not a state")
	    }
    }
  }

  **
  ** Build the menu bar
  **
  Menu makeMenuBar()
  {
    return Menu
    {
      Menu
      {
        text = "File";
        Menu
        {
          text = "New"
          // with no path or name you will be prompted for a name
          // and the path will default to the project directory
          // the first arg true indicates that this is a new diagram not loading
          // one from disk
          MenuItem { text = "State Diagram"; onAction.add {openStateDiagram(true,null,null)} },
        },
//      MenuItem { text = "Back";    image = backIcon;    onAction.add {browser.back} },
//      MenuItem { text = "Next";    image = nextIcon;    onAction.add {browser.forward} },
        MenuItem { text = "Open";  onAction.add |Event e| { openAction(e) } },
        MenuItem { text = "Close";  onAction.add |Event e| { closeAction(e) } },
        MenuItem { text = "Save";    image = saveIcon;    onAction.add {saveAction} },
        MenuItem { text = "Save As...";    image = saveIcon;    onAction.add |Event e| {saveAsAction(e)} },
        MenuItem { text = "Import";    onAction.add {browser.stop} },
        MenuItem { text = "Export";    onAction.add {browser.stop} },
        MenuItem { text = "Exit"; onAction.add |->| { Env.cur.exit } },
      },

      Menu
      {
        text = "Edit";
        MenuItem { text = "Delete";    image = stopIcon;    onAction.add {browser.stop} },
        MenuItem { text = "Undo";      image = undoIcon; onAction.add {undoAction()} },
        MenuItem { text = "Redo";      image = redoIcon; onAction.add {redoAction()} },
      },


      Menu
      {
        text = "View"
        MenuItem { text = "Events"; accelerator=Key.f5; onAction.add{viewEvents()} },
        MenuItem { text = "Full Screen"; accelerator=Key.f1; mode = MenuItemMode.check; onAction.add(cb) },
      },

      Menu
      {
        text = "Help"
        MenuItem { text = "Help"; onAction.add |Event e| { echo(Dialog.openInfo(e.window, "Help Not Yet Written!")) } },
      },

    }
  }

  Void viewEvents()
  {
    if ( this.currentDiagram != null)
    {
      this.currentDiagram.attributes.viewEventsWindow()
    }
  }
  
  Void undoAction()
  {
    if ( this.currentDiagram != null)
    {
      this.currentDiagram.undoAction()
    }
  }
  
  Void redoAction()
  {
    if ( this.currentDiagram != null)
    {
      this.currentDiagram.redoAction()
    }
  }
  
  Void saveAction()
  {
    if ( this.currentDiagram != null)
    {
      this.currentDiagram.saveAction()
    }
    this.eventRegistry.saveChanges()
  }
  
  Void saveAsAction(Event ev)
  {
    if ( this.currentDiagram != null)
    {
      echo("=====")
      echo(this.currentDiagram.settings.diagramDirObj.osPath)
      echo(this.currentDiagram.settings.diagramFile)
      echo("****")
      if ( this.mainWindow == null )
      {
        echo("null window!!!")
      }
      File f:=FileDialog { 
          name=this.currentDiagram.settings.diagramFile;
          dir=this.currentDiagram.settings.diagramDirObj;
          mode=FileDialogMode.saveFile         
      }.open(ev.window)
      
      //this.currentDiagram.saveAction()
    }
    this.eventRegistry.saveChanges()
  }
  
  **
  ** Build the menu bar
  **
  Menu oldmakeMenuBar()
  {
    return Menu
    {
      Menu
      {
        text = "File";
//      MenuItem { text = "Back";    image = backIcon;    onAction.add {browser.back} },
//      MenuItem { text = "Next";    image = nextIcon;    onAction.add {browser.forward} },
//      MenuItem { text = "Refresh"; image = refreshIcon; onAction.add {browser.refresh} },
//      MenuItem { text = "Stop";    image = stopIcon;    onAction.add {browser.stop} },
        MenuItem { text = "Save";    image = saveIcon;    onAction.add {browser.stop} },
        MenuItem { text = "Save As...";    image = saveIcon;    onAction.add {browser.stop} },
        MenuItem { text = "Import";    onAction.add {browser.stop} },
        MenuItem { text = "Export";    onAction.add {browser.stop} },
        MenuItem { text = "Exit"; onAction.add |->| { Env.cur.exit } },
      },

      Menu
      {
        text = "Nested";
        Menu
        {
          text = "Alpha"
          image = folderIcon
          MenuItem { text = "Alpha.1"; onAction.add(cb) },
          Menu
          {
            text = "Alpha.2"
            MenuItem { text = "Alpha.2.I"; onAction.add(cb) },
            Menu
            {
              text = "Alpha.2.II"
              MenuItem { text = "Alpha.2.II.a"; onAction.add(cb) },
              MenuItem { text = "Alpha.2.II.b"; onAction.add(cb) },
            },
            MenuItem { text = "Alpha.2.III"; onAction.add(cb) },
          },
        },
        Menu
        {
          text = "Beta"
          MenuItem { text = "Beta.1"; onAction.add(cb) },
          MenuItem { text = "Beta.2"; onAction.add(cb) },
        },
      },

      Menu
      {
        text = "Modes"
        MenuItem { text = "Check 1"; accelerator=Key.f1; mode = MenuItemMode.check; onAction.add(cb) },
        MenuItem { text = "Check 2"; accelerator=Key.f2; mode = MenuItemMode.check; onAction.add(cb) },
        MenuItem { mode = MenuItemMode.sep },
        MenuItem { text = "Radio 1"; accelerator=Key.num1+Key.alt; mode = MenuItemMode.radio; onAction.add(cb) },
        MenuItem { text = "Radio 2"; accelerator=Key.alt+Key.num2; mode = MenuItemMode.radio; onAction.add(cb); selected = true; text = "Radio_1"  },
      },

      Menu
      {
        text = "Dialogs"
        MenuItem { text = "Info"; onAction.add |Event e| { echo(Dialog.openInfo(e.window, "Test information!")) } },
        MenuItem { text = "Warn"; onAction.add |Event e| { echo(Dialog.openWarn(e.window, "Test warning!")) } },
        MenuItem { text = "Err"; onAction.add |Event e| { echo(Dialog.openErr(e.window, "Test error!")) } },
        MenuItem { text = "Question"; onAction.add |Event e| { echo(Dialog.openQuestion(e.window, "Test question?")) } },
        MenuItem { mode = MenuItemMode.sep },
        MenuItem { text = "Ok/Cancel"; onAction.add |Event e| { echo(Dialog.openInfo(e.window, "OK/Cancel", Dialog.okCancel)) } },
        MenuItem { text = "Yes/No"; onAction.add |Event e| { echo(Dialog.openInfo(e.window, "Yes/No", Dialog.yesNo)) } },
        MenuItem { mode = MenuItemMode.sep },
        MenuItem { text = "Details Err"; onAction.add |Event e| { echo(Dialog.openErr(e.window, "Something bad", ArgErr())) } },
        MenuItem { mode = MenuItemMode.sep },
        MenuItem { text = "Prompt Str 1"; onAction.add |Event e| { echo("--> " + Dialog.openPromptStr(e.window, "Enter a string:")) } },
        MenuItem { text = "Prompt Str 2"; onAction.add |Event e| { echo("--> " + Dialog.openPromptStr(e.window, "Enter a string:", "123", 4)) } },
        MenuItem { mode = MenuItemMode.sep },
        MenuItem { text = "Option A"; onAction.add |Event e| { echo((Dialog(e.window) {body="Str message"; commands=[Dialog.ok]}).open) } },
        MenuItem { text = "Option B"; onAction.add |Event e| { echo((Dialog(e.window) {body=Button { text="BIG!" }; commands=Dialog.okCancel}).open) } },
        MenuItem { mode = MenuItemMode.sep },
        MenuItem { text = "File Open";  onAction.add |Event e| { echo(FileDialog {}.open(e.window)) } },
        MenuItem { text = "Files Open"; onAction.add |Event e| { echo(FileDialog { dir=Env.cur.homeDir; mode=FileDialogMode.openFiles }.open(e.window)) } },
        MenuItem { text = "File Save";  onAction.add |Event e| { echo(FileDialog { name="foo.txt";  mode=FileDialogMode.saveFile }.open(e.window)) } },
        MenuItem { text = "Dir Open";   onAction.add |Event e| { echo(FileDialog { dir=Env.cur.homeDir; mode=FileDialogMode.openDir }.open(e.window)) } },
      },

    }
  }

  **
  ** Build the toolbar
  **
  Widget makeStatusBar()
  {
    statusBar := Label
    {
      //halign = Halign.fill;
      text = "Welcome to JSM Wonderland";
    }
    return(statusBar) 
  }

//  Int cornerRounding()
//  {
//    return stateMachineCanvas.cornerRounding;
//  }
//  
//    
//  Int minStateW()
//  {
//    return stateMachineCanvas.minStateW;
//  }
//  
//  Int minStateH()
//  {
//    return stateMachineCanvas.minStateH;
//  }
//  
//  Color cornerColor()
//  {
//    return stateMachineCanvas.cornerColor;
//  }
//  
//  Color stateColor()
//  {
//    return stateMachineCanvas.stateColor;
//  }
//  
//  Int cornerSize()
//  {
//    return stateMachineCanvas.cornerSize;
//  }
//  
//  Int pseudoCornerSize()
//  {
//    return stateMachineCanvas.pseudoCornerSize;
//  }
//  
  
  
  Void setStatus(Str msg)
  {
    statusBar.text = msg
  }
  
  
  **
  ** Build the toolbar
  **
  Widget makeToolBar()
  {
    return ToolBar
    {
//    Button { image = backIcon;    onAction.add {browser.back} },
//    Button { image = nextIcon;    onAction.add {browser.forward} },
//    Button { image = refreshIcon; onAction.add {browser.refresh} },
//    Button { image = stopIcon;    onAction.add {browser.stop} },
//    Button { mode  = ButtonMode.sep },
//    Button { image = sysIcon;   mode = ButtonMode.check; onAction.add(cb) },
//    Button { image = prefsIcon; mode = ButtonMode.toggle; onAction.add(cb) },
      
//    Button { image = classIcon;    onAction.add{stateMachineCanvas.mode=EditMode.ADD_CLASS;} }, 
      
      Button { mode  = ButtonMode.sep },
      //Button { image = forkIcon; mode = ButtonMode.radio; onAction.add{stateMachineCanvas.mode=EditMode.ADD_FORK;} },
      initialButton,
      finalButton,
      choiceButton,
      junctionButton,
      joinButton,
      forkButton,
      cursorButton,
      transitionButton,
      stateButton,
      Button { mode  = ButtonMode.sep },
      Button { mode  = ButtonMode.sep },
      Button { mode  = ButtonMode.sep },
      Button { image = alignCenterIcon; onAction.add {evPerformAlignButtonClick(AlignMode.CENTER);} },
      Button { image = alignMiddleIcon; onAction.add {evPerformAlignButtonClick(AlignMode.MIDDLE);} },
      Button { image = alignRightIcon;  onAction.add {evPerformAlignButtonClick(AlignMode.RIGHT);} },
      Button { image = alignLeftIcon;   onAction.add {evPerformAlignButtonClick(AlignMode.LEFT);} },
      Button { image = alignTopIcon;    onAction.add {evPerformAlignButtonClick(AlignMode.TOP);} },
      Button { image = alignBottomIcon; onAction.add {evPerformAlignButtonClick(AlignMode.BOTTOM);} },
      Button { mode  = ButtonMode.sep },
      Button { mode  = ButtonMode.sep },
      Button { mode  = ButtonMode.sep },
      undoButton,
      redoButton,
    }
  }
  
  Void evPerformAlignButtonClick(AlignMode alignMode)
  {
    if ( currentDiagram != null )
    {
     currentDiagram.performAlign(alignMode); 
     currentDiagram.checkRedraw();
    }
  }

  **
  ** Build a simple web browser
  **
  Widget makeWebBrowser()
  {
    url := Text { text=homeUri }
    url.onAction.add |->| { browser.load(url.text.toUri) }

    return EdgePane
    {
      top = EdgePane { center=url; right=Label{text="Enter to Go!"} }
      center = browser
    }
  }

  **
  ** Build a pane of various labels
  **
  Widget makeLabels()
  {
    return GridPane
    {
      numCols = 2
      hgap = 20
      halignCells = Halign.fill
      Label { text = "Text Only" },
      Label { image = stopIcon },
      Label { text = "Both"; image = folderIcon },
      Label { text = "Monospace"; font = Desktop.sysFontMonospace },
      Label { text = "Colors"; image = folderIcon; fg = Color.red; bg = Color.yellow },
      Label { text = "Left"; halign = Halign.left },
      Label { text = "Center"; halign = Halign.center },
      Label { text = "Right"; halign = Halign.right },
    }
  }

  **
  ** Build a pane of various progress bars
  **
  Widget makeProgressBar()
  {
    return GridPane
    {
      numCols = 1
      hgap = 20
      halignCells = Halign.fill
      ProgressBar { val=25; },
      ProgressBar { min=0; max=100; val=75; },
      ProgressBar { min=-100; max=100; val=80; },
      ProgressBar { min=-100; max=100; val=25; },
      ProgressBar { indeterminate = true },
    }
  }

  **
  ** Build a pane of various buttons
  **
  Widget makeButtons()
  {
    return GridPane
    {
      numCols = 3
      hgap = 20
      Button { text = "B1"; image = stopIcon; onAction.add(cb) },
      Button { text = "Monospace"; font = Desktop.sysFontMonospace; onAction.add(cb) },
      Button { mode = ButtonMode.toggle; text = "Button 3"; onAction.add(cb) },
      Button { mode = ButtonMode.check; text = "B4"; onAction.add(cb) },
      Button { mode = ButtonMode.radio; text = "Button 5"; onAction.add(cb) },
      Button { mode = ButtonMode.radio; text = "B6"; onAction.add(cb) },
      Button { text = "Popup 1"; onAction.add {JsmGui.popup(true, it)} },
      Button { text = "Popup 2"; onAction.add {JsmGui.popup(false, it)} },
      Button { text = "Disabled"; enabled=false },
      Button { text = "Invisible"; visible=false },
    }
  }
  
  Void evSetEditModeButtonClick(EditMode mode)
  {
    if ( currentDiagram != null  )
    {
      currentDiagram.setEditMode(mode)
      currentDiagram.checkRedraw()
    }
    else
    {
      echo("No diagram selected!")
    }
  }
  

  **
  ** Build a pane of various text fields
  **
  Widget makeText()
  {
    area := Text
    {
      multiLine = true
      font = Desktop.sysFontMonospace
      text ="Press button above to serialize this entire demo here"
    }

    ecb := |Event e| { echo("onAction: \"${e.widget->text}\"") }
    ccb := |Event e| 
    { 
      //echo("onModify: \"${e.widget->text}\"") 
    }

    nums := ["One", "Two", "Three", "Four", "Five", "Six", "Seven" ]

    return EdgePane
    {
      left = GridPane
      {
        numCols = 2

        Label { text="Single" },
        Text { onAction.add(ecb); onModify.add(ccb) },

        Label { text="Monospace";  },
        Text { font = Desktop.sysFontMonospace; onAction.add(ecb); onModify.add(ccb)  },

        Label { text="Password" },
        Text { password = true; onAction.add(ecb); onModify.add(ccb) },

        Label { text="Combo" },
        Combo { items=nums; onAction.add(ecb); onModify.add(ccb) },

        Label { text="Combo editable=true" },
        Combo { editable=true; items=nums; onAction.add(ecb); onModify.add(ccb) },

        Label { text="Combo dropDown=false" },
        Combo { dropDown=false; items=nums; onAction.add(ecb); onModify.add(ccb) },

        Label { text="MultiLine" },

        Button { text="Serialize Demo"; onAction.add {serializeTo(area)} },
      }
      center = InsetPane.make(5) { content=area }
    }
  }

  Void serializeTo(Text area)
  {
    try
    {
      opts := ["indent":2, "skipDefaults":true, "skipErrors":true]
      buf := Buf.make.writeObj(area.window, opts)
      area.text = buf.flip.readAllStr
    }
    catch (Err e)
    {
      area.text = e.traceToStr
    }
  }

  **
  ** Build a demo border pane
  **
  Widget makeBorderPane()
  {
    b := BorderPane
    {
      border = Border.defVal
      insets = Insets(10)
      content = Box { color = Color.blue }
    }

    borderText := Text { text = b.border.toStr }
    insetsText := Text { text = b.insets.toStr }
    bgText     := Text { text = "" }

    update := |->|
    {
      b.border = Border(borderText.text)
      b.insets = Insets(insetsText.text)
      b.bg     = bgText.text.isEmpty ? null : Color(bgText.text)
      b.relayout
      b.repaint
    }

    borderText.onAction.add(update)
    insetsText.onAction.add(update)
    bgText.onAction.add(update)

    controlPane := GridPane
    {
      numCols = 2
      Label { text="border" }, borderText,
      Label { text="insets" }, insetsText,
      Label { text="bg" }, bgText,
      Button { text = "Update"; onAction.add(update) }
    }

    return EdgePane
    {
      left   = controlPane
      center = BorderPane { bg = Color.white; insets = Insets(10); content = b }
    }
  }

  **
  ** Build a demo edge pane
  **
  Widget makeEdgePane()
  {
    return EdgePane
    {
      top    = Button { text = "top" }
      left   = Button { text = "left" }
      right  = Button { text = "right" }
      bottom = Button { text = "bottom" }
      center = Button { text = "center" }
    }
  }

  **
  ** Build a demo grid pane using randomly sized boxes
  **
  Widget makeGridPane()
  {
    grid := GridPane
    {
      numCols = 5
      hgap = 10
      vgap = 10
      Box { color = Color.red },
      Box { color = Color.green },
      Box { color = Color.yellow },
      Box { color = Color.blue },
      Box { color = Color.orange },
      Box { color = Color.darkGray },
      Box { color = Color.purple },
      Box { color = Color.gray },
      Box { color = Color.white },
    }
    colors := [Color.red, Color.green, Color.yellow, Color.blue, Color.orange,
               Color.darkGray, Color.purple, Color.gray, Color.white]

    15.times |Int i| { grid.add(Box { color=colors[i%colors.size] }) }

    controls := GridPane
    {
      numCols = 2
      halignCells = Halign.fill
      Label { text="numCols" },      Text { text="5"; onModify.add {setInt(grid, "numCols", it)} },
      Label { text="hgap" },         Text { text="10"; onModify.add {setInt(grid, "hgap", it)} },
      Label { text="vgap" },         Text { text="10"; onModify.add {setInt(grid, "vgap", it)} },
      Label { text="halignCells" },  Combo { items=Halign.vals; onModify.add {setEnum(grid, "halignCells", it)} },
      Label { text="valignCells" },  Combo { items=Valign.vals; onModify.add {setEnum(grid, "valignCells", it)} },
      Label { text="halignPane" },   Combo { items=Halign.vals; onModify.add {setEnum(grid, "halignPane", it)} },
      Label { text="valignPane" },   Combo { items=Valign.vals; onModify.add {setEnum(grid, "valignPane", it)} },
      Label { text="expandRow" },    Text { text="null"; onModify.add {setInt(grid, "expandRow", it)} },
      Label { text="expandCol" },    Text { text="null"; onModify.add {setInt(grid, "expandCol", it)} },
      Label { text="uniformCols" },  Combo { items=[false,true]; onModify.add {setBool(grid, "uniformCols", it)} },
      Label { text="uniformRows" },  Combo { items=[false,true]; onModify.add {setBool(grid, "uniformRows", it)} },
    }

    return EdgePane { left=controls; center=InsetPane { content=grid } }
  }

  **
  ** Build a demo tree and table for file system
  **
  Widget makeTreeAndTable()
  {
    tree := Tree
    {
      multi = true
      model = DirTreeModel { demo = this }
      onAction.add |Event e| { echo(e) }
      onSelect.add |Event e| { echo(e); echo("selected=${e->widget->selected}") }
      onPopup.add |Event e|  { echo(e); e.popup = makePopup }
      // onMouseMove.add |Event e| { echo(e.pos + ": " + e->widget->nodeAt(e.pos)) }
      // hbar.onModify.add(&onScroll("Tree.hbar"))
      // vbar.onModify.add(&onScroll("Tree.vbar"))
    }

    table := Table
    {
      multi = true
      model = DirTableModel { demo = this; dir = File.os(".").list }
      onAction.add |Event e| { echo(e) }
      onSelect.add |Event e| { echo(e); echo("selected=${e->widget->selected}") }
      onPopup.add |Event e|  { echo(e); e.popup = makePopup }
      // onMouseMove.add |Event e| { Int? row := e->widget->rowAt(e.pos); Int? col := e->widget->colAt(e.pos); echo("Row: $row Col: $col " + ((row != null && col != null) ? e->widget->model->text(col, row) : "")) }
      // hbar.onModify.add(&onScroll("Table.hbar"))
      // vbar.onModify.add(&onScroll("Table.vbar"))
    }

    updateTable := |File dir| { table.model->dir = dir.list; table.refreshAll }
    tree.onAction.add  |Event e| { updateTable(e.data) }
    table.onAction.add |Event e| { updateTable(table.model->dir->get(e.index)) }

    return SashPane
    {
      weights = [1,3]
      tree,
      table,
    }
  }

  **
  ** Build a pane showing how the various window options work
  **
  Widget makeWindow()
  {
    mode := Combo { items = WindowMode.vals; editable=false }
    alwaysOnTop := Button { it.mode = ButtonMode.check; text = "alwaysOnTop" }
    resizable := Button { it.mode = ButtonMode.check; text = "resizable" }
    showTrim := Button { it.mode = ButtonMode.check; text = "showTrim"; selected = true }

    open := |->|
    {
      close := Button { text="Close Me" }
      w := Window(mode.window)
      {
        it.mode = mode.selected
        it.alwaysOnTop = alwaysOnTop.selected
        it.resizable = resizable.selected
        it.showTrim = showTrim.selected
        it.size = Size(200,200)
        GridPane { halignPane = Halign.center; valignPane = Valign.center; add(close) },
      }
      close.onAction.add { w.close }
      w.open
    }

    return GridPane
    {
      mode,
      alwaysOnTop,
      resizable,
      showTrim,
      Button { text="Open"; onAction.add(open) },
    }
  }

  **
  ** Build a pane showing how to use serialization
  **
  Widget makeSerialization()
  {
    area := Text
    {
      multiLine = true
      font = Desktop.sysFontMonospace
      text =
        "fwt::EdgePane\n" +
        "{\n" +
        "  top = fwt::Button { text=\"Top\" }\n" +
        "  center = fwt::Button { text=\"Center\" }\n" +
        "  bottom = fwt::Button { text=\"Bottom\" }\n" +
        "}\n"
    }

    test := InsetPane
    {
      Label { text="Press button to deserialize code on the left here" },
    }

    return SashPane
    {
      EdgePane
      {
        center = area
        right = InsetPane
        {
          Button { text="=>"; onAction.add |->| { deserializeTo(area.text, test) } },
        }
      },
      test,
    }
  }

  Void deserializeTo(Str text, InsetPane test)
  {
    try
    {
      test.content = text.in.readObj
    }
    catch (Err e)
    {
      test.content = Text { it.multiLine = true; it.text = e.traceToStr }
    }
    test.relayout
  }

  **
  ** Build a pane to draw state diagrams
  **

  **
  ** Build a pane showing how to use Graphics
  **
  Widget makeGraphics()
  {
    return ScrollPane { content=GraphicsDemo { demo = this } }
  }

  static Void setInt(Widget obj, Str field, Event e)
  {
    f := obj.typeof.field(field)
    Str text := e.widget->text
    int := text.toInt(10, false)
    if (int != null || text=="null") f.set(obj, int)
    obj.relayout
  }

  static Void setBool(Widget obj, Str field, Event e)
  {
    f := obj.typeof.field(field)
    Str text := e.widget->text
    b := text.toBool(false)
    if (b != null) f.set(obj, b)
    obj.relayout
  }

  static Void setEnum(Widget obj, Str field, Event e)
  {
    f := obj.typeof.field(field)
    en := f.get(obj)->fromStr(e.widget->text, false)
    if (en != null) f.set(obj, en)
    obj.relayout
  }

  static |Event e| cb()
  {
    return |Event e|
    {
      w := e.widget
      echo("${w->text} selected=${w->selected}")
    }
  }

//  |Event e| cursorMode()
//  {
//    return |Event e|
//    {
//      w := e.widget
//      this.currentButton=this.cursorButton;
//      stateMachineCanvas.mode = EditMode.ARROW 
//      stateMachineCanvas.currentNode = null
//      //echo("xxx ${w->text} selected=${w->selected}")
//      stateMachineCanvas.repaint
//    }
//  }
  
//  Void startTransition()
//  {
//      //setMode(EditMode.CONNECT)
//      setMode(EditMode.CONNECT);
//      //stateMachineCanvas.mode = EditMode.CONNECT 
//      stateMachineCanvas.currentNode = null
//      stateMachineCanvas.selectedNodes.clear()
//      echo("*** Starting transition ***") 
//      //echo("xxx ${w->text} selected=${w->selected}")
//      //stateMachineCanvas.repaint
//  }

  static Void popup(Bool withPos, Event event)
  {
    makePopup.open(event.widget, withPos ? Point.make(0, event.widget.size.h) : event.pos)
  }

  static Menu makePopup()
  {
    return Menu
    {
      MenuItem { text = "Popup 1"; onAction.add(cb) },
      MenuItem { text = "Popup 2"; onAction.add(cb) },
      MenuItem { text = "Popup 3"; onAction.add(cb) },
    }
  }

  static Void onScroll(Str name, Event e)
  {
    ScrollBar sb := e.widget
    echo("-- onScroll $name $e  [val=$sb.val min=$sb.min max=$sb.max thumb=$sb.thumb page=$sb.page orient=$sb.orientation")
  }

  WebBrowser browser := WebBrowser {}
  Str homeUri := "http://fantom.org/"

  //File scriptDir  := File.make(this.typeof->sourceFile.toStr.toUri).parent
  
//  Str imageDir:= "file:///d:/Profiles/p56391/f4workspace/JsmGui/fan/images"
  Str imageDir:="file:///c:/Users/Joe/f4workspace/JMT/fan/images"
//  File imagePath:=Uri("#{imageDir}/").toFile()
//  if ( imagePath.exists )
//  {
//      imageDir="file:///c:/Users/Joe/f4workspace/JMT/fan/images"
//  }   

//  Image backIcon       := Image(`fan://icons/x16/arrowLeft.png`)
//  Image nextIcon       := Image(`fan://icons/x16/arrowRight.png`)
  Image cutIcon        := Image(`fan://icons/x16/cut.png`)
  Image copyIcon       := Image(`fan://icons/x16/copy.png`)
  Image pasteIcon      := Image(`fan://icons/x16/paste.png`)
  Image saveIcon      := Image(`fan://icons/x16/save.png`)
  Image settingsIcon      := Image(`fan://icons/x16/settings.png`)
    Image folderIcon     := Image(`fan://icons/x16/folder.png`)
    Image fileIcon       := Image(`fan://icons/x16/file.png`)
    Image audioIcon      := Image(`fan://icons/x16/file.png`)
//  Image classIcon      := Image:(`${imageDir}/jsm_class.png`)
  Image stateIcon      := Image(`${imageDir}/jsm_state_s.png`)
  Image initialIcon    := Image(`${imageDir}/jsm_initial.png`)
  Image finalIcon      := Image(`${imageDir}/jsm_final.png`)
  Image choiceIcon     := Image(`${imageDir}/jsm_choice.png`)
  Image joinIcon       := Image(`${imageDir}/jsm_join.png`)
  Image junctionIcon   := Image(`${imageDir}/jsm_junction.png`)
  Image forkIcon       := Image(`${imageDir}/jsm_fork.png`)
  Image transitionIcon := Image(`${imageDir}/jsm_transition_arrow.png`)
  Image cursorIcon     := Image(`${imageDir}/cursor.png`)
  //Image transitionIcon   := Image(`${imageDir}///c:/icons/x16/silk/icons/arrow_right.png`)
  Image alignLeftIcon  := Image(`${imageDir}/shape_align_left.png`)
  Image alignRightIcon := Image(`${imageDir}/shape_align_right.png`)
  Image alignTopIcon   := Image(`${imageDir}/shape_align_top.png`)
  Image alignBottomIcon:= Image(`${imageDir}/shape_align_bottom.png`)
  Image alignCenterIcon:= Image(`${imageDir}/shape_align_center.png`)
  Image alignMiddleIcon:= Image(`${imageDir}/shape_align_middle.png`)
  Image sysIcon        := Image(`fan://icons/x16/file.png`)
  Image prefsIcon      := Image(`fan://icons/x16/file.png`)
  Image refreshIcon    := Image(`fan://icons/x16/refresh.png`)
  Image undoIcon    := Image(`fan://icons/x16/undo.png`)
  Image redoIcon    := Image(`fan://icons/x16/redo.png`)
  Image stopIcon       := Image(`fan://icons/x16/err.png`)
  Image cloudIcon      := Image(`fan://icons/x16/cloud.png`)
  
  Button initialButton    := Button { image = initialIcon; mode = ButtonMode.radio; onAction.add{evSetEditModeButtonClick(EditMode.ADD_INITIAL);} }
  Button finalButton      := Button { image = finalIcon; mode = ButtonMode.radio; onAction.add{evSetEditModeButtonClick(EditMode.ADD_FINAL);} }
  Button choiceButton     := Button { image = choiceIcon; mode = ButtonMode.radio; onAction.add{evSetEditModeButtonClick(EditMode.ADD_CHOICE);} }
  Button junctionButton   := Button { image = junctionIcon; mode = ButtonMode.radio; onAction.add{evSetEditModeButtonClick(EditMode.ADD_JUNCTION);} }
  Button joinButton       := Button { image = joinIcon; mode = ButtonMode.radio; onAction.add{evSetEditModeButtonClick(EditMode.ADD_JOIN);} }
  Button forkButton       := Button { image = forkIcon; mode = ButtonMode.radio; onAction.add{evSetEditModeButtonClick(EditMode.ADD_FORK);} }
  Button cursorButton     := Button { image = cursorIcon; mode = ButtonMode.radio; onAction.add{evSetEditModeButtonClick(EditMode.ARROW);} }
  Button transitionButton := Button { image = transitionIcon; mode = ButtonMode.radio; onAction.add{evSetEditModeButtonClick(EditMode.CONNECT);} }
  Button stateButton      := Button { image = stateIcon;    mode=ButtonMode.radio; onAction.add {evSetEditModeButtonClick(EditMode.ADD_STATE);} }
  Button redoButton      := Button { image = redoIcon;    mode=ButtonMode.radio; onAction.add {undoAction();} }
  Button undoButton      := Button { image = undoIcon;    mode=ButtonMode.radio; onAction.add {undoAction();} }
  
}

**************************************************************************
** DirTreeModel
**************************************************************************

class DirTreeModel : TreeModel
{
  JsmGui? demo

  override Obj[] roots() { return Env.cur.homeDir.listDirs }

  override Str text(Obj node) { return node->name }

  override Image? image(Obj node) { return demo.folderIcon }

  override Obj[] children(Obj obj) { return obj->listDirs }
}

**************************************************************************
** DirTableModel
**************************************************************************

class DirTableModel : TableModel
{
  JsmGui? demo
  File[]? dir
  Str[] headers := ["Name", "Size", "Modified"]
  override Int numCols() { return 3 }
  override Int numRows() { return dir.size }
  override Str header(Int col) { return headers[col] }
  override Halign halign(Int col) { return col == 1 ? Halign.right : Halign.left }
  override Font? font(Int col, Int row) { return col == 2 ? Font {name=Desktop.sysFont.name; size=Desktop.sysFont.size-1} : null }
  override Color? fg(Int col, Int row)  { return col == 2 ? Color("#666") : null }
  override Color? bg(Int col, Int row)  { return col == 2 ? Color("#eee") : null }
  override Str text(Int col, Int row)
  {
    f := dir[row]
    switch (col)
    {
      case 0:  return f.name
      case 1:  return f.size?.toLocale("B") ?: ""
      case 2:  return f.modified.toLocale
      default: return "?"
    }
  }
  override Int sortCompare(Int col, Int row1, Int row2)
  {
    a := dir[row1]
    b := dir[row2]
    switch (col)
    {
      case 1:  return a.size <=> b.size
      case 2:  return a.modified <=> b.modified
      default: return super.sortCompare(col, row1, row2)
    }
  }
  override Image? image(Int col, Int row)
  {
    if (col != 0) return null
    return dir[row].isDir ? demo.folderIcon : demo.fileIcon
  }
}

**************************************************************************
** Box
**************************************************************************

class Box : Canvas
{
  Color color := Color.green

  override Size prefSize(Hints hints := Hints.defVal)
  {
    Size(Int.random(20..100), Int.random(20..80))
  }

  override Void onPaint(Graphics g)
  {
    size := this.size
    g.brush = color
    g.fillRect(0, 0, size.w, size.h)
    g.brush = Color.black
    g.drawRect(0, 0, size.w-1, size.h-1)
  }
}

**************************************************************************
** StateMachineCanvas
**************************************************************************


**************************************************************************
** GraphicsDemo
**************************************************************************

class GraphicsDemo : Canvas
{
  JsmGui? demo

  override Size prefSize(Hints hints := Hints.defVal) { return Size.make(750, 450) }

  override Void onPaint(Graphics g)
  {
    w := size.w
    h := size.h

    g.antialias = true

    g.brush = Gradient("0% 0%, 100% 100%, #fff, #666")
    g.fillRect(0, 0, w, h)

    g.brush = Color.black; g.drawRect(0, 0, w-1, h-1)

    g.brush = Color.orange; g.fillRect(10, 10, 50, 60)
    g.brush = Color.blue; g.drawRect(10, 10, 50, 60)

    g.brush = Color("#80ffff00"); g.fillOval(40, 40, 120, 100)
    g.pen = Pen { width = 2; dash=[8,4].toImmutable }
    g.brush = Color.green; g.drawOval(40, 40, 120, 100)

    g.pen = Pen { width = 8; join = Pen.joinBevel }
    g.brush = Color.gray; g.drawRect(120, 120, 120, 90)
    g.brush = Color.orange; g.fillArc(120, 120, 120, 90, 45, 90)
    g.pen = Pen { width = 8; cap = Pen.capRound }
    g.brush = Color.blue; g.drawArc(120, 120, 120, 90, 45, 90)

    g.brush = Color.purple; g.drawText("Hello World!", 70, 50)
    g.font = Desktop.sysFontMonospace.toSize(16).toBold; g.drawText("Hello World!", 70, 70)

    g.pen = Pen { width = 2; join = Pen.joinBevel }
    g.brush = Color("#a00")
    g.drawPolyline([
      Point(10, 380),
      Point(30, 420),
      Point(50, 380),
      Point(70, 420),
      Point(90, 380)])

    polygon := [Point(180, 380), Point(140, 440), Point(220, 440)]
    g.pen = Pen("1")
    g.brush = Color("#f88"); g.fillPolygon(polygon)
    g.brush = Color("#800"); g.drawPolygon(polygon)

    img := demo.folderIcon
    g.drawImage(img, 220, 20)
    g.copyImage(img, Rect(0, 0, img.size.w, img.size.h), Rect(250, 30, 64, 64))
    g.drawImage(img.resize(Size(64, 64)), 320, 30)
    g.push
    try
    {
      g.alpha=128; g.drawImage(img, 220, 40)
      g.alpha=64;  g.drawImage(img, 220, 60)
    }
    finally g.pop

    // image brush
    g.brush = Pattern(demo.cloudIcon)
    g.fillOval(390, 20, 80, 80)
    g.brush = Color.black
    g.pen = Pen { width = 1 }
    g.drawOval(390, 20, 80, 80)

    // system font/colors
    y := 20
    g.brush = Color.black
    g.font = Desktop.sysFont
    g.drawText("sysFont: $Desktop.sysFont.toStr", 480, y)
    g.font = Desktop.sysFontSmall
    g.drawText("sysFontSmall: $Desktop.sysFontSmall.toStr", 480, y+18)
    g.font = Desktop.sysFontView
    g.drawText("sysFontView: $Desktop.sysFontView.toStr", 480, y+30)
    y += 60
    g.font = Font("9pt Arial")
    y = sysColor(g, y, Desktop.sysDarkShadow, "sysDarkShadow")
    y = sysColor(g, y, Desktop.sysNormShadow, "sysNormShadow")
    y = sysColor(g, y, Desktop.sysLightShadow, "sysLightShadow")
    y = sysColor(g, y, Desktop.sysHighlightShadow, "sysHighlightShadow")
    y = sysColor(g, y, Desktop.sysFg, "sysFg")
    y = sysColor(g, y, Desktop.sysBg, "sysBg")
    y = sysColor(g, y, Desktop.sysBorder, "sysBorder")
    y = sysColor(g, y, Desktop.sysListBg, "sysListBg")
    y = sysColor(g, y, Desktop.sysListFg, "sysListFg")
    y = sysColor(g, y, Desktop.sysListSelBg, "sysListSelBg")
    y = sysColor(g, y, Desktop.sysListSelFg, "sysListSelFg")

    // rect/text with gradients
    g.brush = Gradient("260px 120px, 460px 320px, #00f, #f00")
    g.pen = Pen { width=20; join = Pen.joinRound }
    g.drawRect(270, 130, 180, 180)
    6.times |Int i| { g.drawText("Gradients!", 300, 150+i*20) }

    // translate for font metric box
    g.translate(50, 250)
    g.pen = Pen.defVal
    g.brush = Color.yellow
    g.fillRect(0, 0, 200, 100)

    // font metric box with ascent, descent, baseline
    g.font = Desktop.sysFont.toSize(20)
    tw := g.font.width("Font Metrics")
    tx := (200-tw)/2 // 200 is width of yellow box, center x in box
    ty := 30 // Down 30 from top of box
    g.brush = Color.gray
    g.drawLine(tx-10, ty, tx+10, ty)  // the cross
    g.drawLine(tx, ty-10, tx, ty+10)  // the cross, intersection is tx
    g.brush = Color.orange
    my := ty+g.font.leading; // white space above letters
    g.drawLine(tx, my, tx+tw, my)
    g.brush = Color.green
    my += g.font.ascent;   // height of letters
    g.drawLine(tx, my, tx+tw, my)
    g.brush = Color.blue  
    my += g.font.descent; // white space below letters
    g.drawLine(tx, my, tx+tw, my)
    g.brush = Color.black
    g.drawText("Font Metrics", tx, ty)

    // alpha
    g.translate(430, 80)
    // checkerboard bg
    g.brush = Color.white
    g.fillRect(0, 0, 240, 120)
    g.brush = Color("#ccc")
    12.times |Int by| {
      24.times |Int bx| {
        if (bx.isEven.xor(by.isEven))
          g.fillRect(bx*10, by*10, 10, 10)
      }
    }
    // change both alpha and color
    a := Color("#ffff0000")
    b := Color("#80ff0000")
    g.alpha=255; g.brush=a; g.fillRect(0, 0,  30, 30); g.brush=b; g.fillRect(30, 0,  30, 30)
    g.alpha=192; g.brush=a; g.fillRect(0, 30, 30, 30); g.brush=b; g.fillRect(30, 30, 30, 30)
    g.alpha=128; g.brush=a; g.fillRect(0, 60, 30, 30); g.brush=b; g.fillRect(30, 60, 30, 30)
    g.alpha=64;  g.brush=a; g.fillRect(0, 90, 30, 30); g.brush=b; g.fillRect(30, 90, 30, 30)
    // change only alpha
    g.brush = a
    g.alpha=255; g.fillRect(60, 0,  30, 30);
    g.alpha=192; g.fillRect(60, 30, 30, 30);
    g.alpha=128; g.fillRect(60, 60, 30, 30);
    g.alpha=64;  g.fillRect(60, 90, 30, 30);
    // change only color
    g.alpha = 128
    g.brush = Color("#f00"); g.fillRect(90, 0,  30, 30);
    g.brush = Color("#ff0"); g.fillRect(90, 30, 30, 30);
    g.brush = Color("#0f0"); g.fillRect(90, 60, 30, 30);
    g.brush = Color("#00f"); g.fillRect(90, 90, 30, 30);
    // gradients
    g.alpha = 255
    g.brush = Gradient("0px 0px, 0px 120px, #f00, #fff");           g.fillRect(120, 0, 20, 120)
    g.brush = Gradient("0px 0px, 0px 120px, #f00, #80ffffff");      g.fillRect(140, 0, 20, 120)
    g.brush = Gradient("0px 0px, 0px 120px, #80ff0000, #80ffffff"); g.fillRect(160, 0, 20, 120)
    g.brush = Gradient("0px 0px, 0px 120px, #f00, #fff");
      g.alpha = 128; /* set alpha after gradient */  g.fillRect(180, 0, 20, 120)
    g.brush = Gradient("0px 0px, 0px 120px, #f00, #80ffffff");      g.fillRect(200, 0, 20, 120)
    g.brush = Gradient("0px 0px, 0px 120px, #80ff0000, #80ffffff"); g.fillRect(220, 0, 20, 120)
  }

  Int sysColor(Graphics g, Int y, Color c, Str name)
  {
    g.brush = c
    g.fillRect(480, y, 140, 20)
    g.brush = Color.green
    g.drawText(name, 490, y+3)
    return y + 20
  }
}
