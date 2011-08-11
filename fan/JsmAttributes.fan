using gfx
using fwt

class JsmAttributes
{
  JsmDiagram diagram
  JsmNode? currentNode
  JsmState? currentState
  JsmConnection? currentConn
  //Str[] eventNames:=Str[,]
  //EventDef[] events:=EventDef[,]
  EventTableModel eventsModel:=EventTableModel()
  Int currentUpdateNo:=0
  Int lastSavedUpdateNo:=0
  //GridPane attrPane
  GridPane diagramSettingsPane
  GridPane statePane
  GridPane eventsPane
  Window eventsWindow
  GridPane? debugStatePane
  GridPane transitionPane
  Text stateName:=Text { onModify.add { if (currentNode!=null){currentNode.name=stateName.text}   } }
  Text regionName:=Text { editable=false; }
  Text eventsList:=Text { multiLine=true; editable=false; }
  Text connName:=Text { onModify.add { if (currentConn!=null){currentConn.name=connName.text}   } }
  Text transitionName:=Text { onModify.add { if (currentConn!=null){currentConn.name=transitionName.text}   } }
  Text entryActivity:=Text { multiLine=true; onModify.add { if (currentState!=null){currentState.entryActivity=entryActivity.text}   } }
  Text exitActivity:=Text { multiLine=true; onModify.add { if (currentState!=null){currentState.exitActivity=exitActivity.text}   } }
  Text doActivity:=Text { multiLine=true; onModify.add { if (currentState!=null){currentState.doActivity=doActivity.text}   } }
  Text trigger:=Text { multiLine=true; onModify.add { if (currentConn!=null){currentConn.event=trigger.text}   } }
  Text guard:=Text { multiLine=true; onModify.add { if (currentConn!=null){currentConn.guard=guard.text}   } }
  Text action:=Text { multiLine=true; onModify.add { if (currentConn!=null){currentConn.action=action.text}   } }
  Text parentState:=Text { editable=false; }
  //Combo eventsCombo := Combo { dropDown=false; items = eventNames; editable = false }
  Table eventsTable := Table { multi=true  }
  Button genStateMachineButton:= Button { text="Generate"; onAction.add { genStateMachine()   } }
  Button saveStateMachineButton:= Button { text="Save Changes"; onAction.add { saveStateMachine()   } }
  Button eventsButton := Button { text="Edit Events"; onAction.add { viewEventsWindow()   } }
  Button eventNewButton := Button { text="New Event";  onAction.add { eventNew()  } }
  Button eventApplyButton := Button { text="Apply";      onAction.add { eventApply()  } }
  Button eventCancelButton := Button { text="Cancel";     onAction.add { eventCancel() } }
  Text x1:=Text { }
  Text y1:=Text { }
  Text x2:=Text { }
  Text y2:=Text { }
  Text regions:=Text { }
  Buf[] lastInc
  Buf[] redoInc
  Text fillColor:=Text { }
  Text internalDetails:=Text { 
       onModify.add { if (currentNode!=null){currentNode.spec=internalDetails.text}   }
       //onModify.add { if (currentConn!=null){currentConn.spec=internalDetails.text}   }
       multiLine = true 
     }
  Text coords:=Text { }
  Text nodeCount:=Text { }
  Text substateCount:=Text { }
  Text region_1:=Text { }
  //JsmState? rootState
  //JsmState? rootNode
  StateMachineCanvas? canvas
//Text { onAction.add(ecb); onModify.add(ccb) },
  
  Text diagramName:=Text { onModify.add { updateDiagramName() } }
  Text rootStateName:=Text { onModify.add { updateRootStateName() } }
  Text diagramPath:=Text { onModify.add { updateDiagramPath() } }
  
  
  new make(JsmDiagram diagram)
  {
    this.diagram=diagram
    this.lastInc=Buf[,]
    this.redoInc=Buf[,]
    
    diagramSettingsPane= GridPane
    {
        numCols = 2
        halignCells=Halign.fill

        Label { text="Diagram Name" },        diagramName,
        Label { text="Root State" },          rootStateName,
        Label { text="Diagram Path" },        diagramPath,
        Label { text="" },                    genStateMachineButton,
        Label { text="" },                    saveStateMachineButton,
    }
    diagramSettingsPane.expandCol=1
    
    
    statePane= GridPane
    {
        numCols = 2
        halignCells=Halign.fill

        Label { text="Name" },           stateName,
        Label { text="Region" },         regionName,
        Label { text="Parent State" },   parentState,
        Label { text="Entry\r\nActivity" },          entryActivity,
        Label { text="Exit\r\nActivity" },          exitActivity,
        Label { text="Internal\r\nEvents" },          
        Button { text="Edit"; onAction.add { editInternalEvents()   } },
        Button { text="Add Region"; onAction.add { evAddRegionButtonClick()   } },
        Button { text="Remove Last Region"; onAction.add { delRegion()   } },
        Label { text="Do\r\nActivity" },          doActivity,
        Label { text="Fill Color" },     fillColor,
    }
    statePane.expandCol=1
    
    
    Menu eventsMenu := Menu
      {
        Menu
        {
	        text = "Event";
	        MenuItem { text = "New";    onAction.add {eventNew} },
	        MenuItem { text = "Edit";   onAction.add {eventEdit} },
	        MenuItem { text = "Delete"; onAction.add {eventDelete} },
	        MenuItem { text = "Close";  onAction.add { eventClose } },
        },
      }

    
//    GridPane eventsPane2 :=  GridPane { 
//          halignPane = Halign.fill; 
//          halignCells=Halign.fill
//          numCols = 3
//          Button { text="New";    onAction.add { newEvent()  } },
//          Button { text="Edit";   onAction.add { editEvent() } },
//          Button { text="Delete"; onAction.add { delEvent()  } },
//    }
    
    GridPane eventsPane1 :=  GridPane { 
          halignPane = Halign.center; 
          halignCells=Halign.fill
          numCols = 3
          expandRow = 0
          eventNewButton,
          eventApplyButton,
          eventCancelButton,
    }
    
    eventsPane =  GridPane { 
          halignPane = Halign.center; 
          valignPane = Valign.center; 
          valignPane = Valign.fill; 
          valignCells = Valign.fill;  
          halignPane = Halign.fill; 
          halignCells=Halign.fill
          expandCol = 0
          expandRow = 0
          numCols = 1
          eventsTable,
          eventsPane1,
    }
    eventsTable.model=eventsModel
    eventsModel.events=this.events()
    
    eventsWindow = Window(this.diagram.gui.mainWindow)
    {
        it.title = "${this.diagram.settings.diagramName} State Machine Events"
        it.mode = WindowMode.windowModal
        it.alwaysOnTop = true
        it.resizable = true
        it.showTrim = true
        it.size = Size(600,400)
        eventsMenu,
        eventsPane,
    }
    //eventsWindow.open    
    
    transitionPane= GridPane
    {
      numCols = 1
      halignCells=Halign.fill
      halignPane=Halign.fill
      expandCol=0
      valignPane=Valign.fill
      GridPane { 
          halignPane = Halign.center; 
          halignCells=Halign.fill
          expandCol=0
          numCols = 1
          Label {  text="Transition"; halign=Halign.center }, connName,
          eventsButton,         eventsList,
          Label {  text="Guard"; halign=Halign.center }, 
          guard,
          Label {  text="Action"; halign=Halign.center }, 
          action,
      },
//        GridPane { 
//          halignPane = Halign.center; 
//          halignCells=Halign.fill
//          numCols = 2
//          Label { text="Guard" },          guard,
//          Label { text="Action" },         action,
//        }
    }
    //transitionPane.expandCol=1
    
    
  }
  
  Void viewEventsWindow()
  {
//  Str x:=this.currentConn.event.replace("\r\n", ",")
//  echo("setting event text to $this.currentConn.event === <$x>")
//    eventsCombo.text=this.currentConn.event.replace("\r\n", ",")
//    echo("set event text to $eventsCombo.text")
//    eventsCombo.text="aaa"
//    eventsCombo.relayout
    if ( this.currentConn != null )
    {
      Int[] evSelected:=[,]
      this.currentConn.event.splitLines.sort.each |evName|
      {
        EventDef? evDef:=this.diagram.gui.eventRegistry.get(evName)     
        if ( evDef != null )
        {
          Int? idx:=events.index(evDef)
          if ( idx != null )
          {
            evSelected.add(idx)   
          }
        }
      }
      eventsTable.selected=evSelected
      eventApplyButton.enabled=true
      eventCancelButton.enabled=true
    }
    else
    {
      eventApplyButton.enabled=false
      eventCancelButton.enabled=false
    }
    
//    eventsTable.refreshAll()
    eventsWindow.relayout
    eventsWindow.open
  }
  
  Void updateDiagramPath()
  {
    echo("Updated diagram path $this.diagramPath.text")
    if ( this.diagramPath.text != "" && this.diagramPath.text != this.diagram.settings.diagramPath )
    {
      // change the diagram settings for the path
      this.diagram.settings.diagramPath=this.diagramPath.text
      currentUpdateNo++
    }
  }
  
  Void updateDiagramName()
  {
    echo("Updated diagram name $this.diagramName.text")
    if ( this.diagramName.text != "" && this.diagramName.text != this.diagram.settings.diagramName )
    {
      // update the tab name
      this.diagram.diagramTab.text=this.diagramName.text
      // change the diagram name in the path to the new name
      this.diagramPath.text=this.diagramPath.text.replace(this.diagram.settings.diagramName, this.diagramName.text)
      // change the diagram settings for the path
      this.diagram.settings.diagramName=this.diagramName.text
      currentUpdateNo++
    }
  }
  
  Void updateRootStateName()
  {
    echo("Updated root state name $this.rootStateName.text")
    if ( this.rootStateName.text != "" && this.rootStateName.text != this.diagram.stateMachineCanvas.rootState.name )
    {
      //this.diagram.stateMachineCanvas.rootState.name = this.rootStateName.text
      this.diagramName.text=this.diagramName.text.replace(this.diagramName.text, this.rootStateName.text)
      this.diagram.stateMachineCanvas.rootState.name = this.rootStateName.text 
      currentUpdateNo++
    }
  }
  
  Void eventApply()
  {
    
    eventList:=this.eventsTable.selected.map |Int idx->Str| {return events[idx].name}
    this.currentConn.event=eventList.join("\r\n")
    this.eventsList.text=this.currentConn.event
    
//    if ( this.eventsCombo.text.trim != "" )
//    {
//      this.currentConn.event=this.eventsCombo.text
//      this.eventsCombo.text.trim.split(',').each |ev|
//      {
//        if ( ! this.eventNames.contains(ev))
//        {
//          this.eventNames.add(ev)
//          this.eventsCombo.items=this.eventNames
//        }
//      }
      //this.eventsList.text=this.eventNames.sort.join("\r\n")
//      this.eventsList.text=this.eventsCombo.text
//      this.eventsPane.repaint
//    }
    this.eventsWindow.close()
  }
  
  Void eventClose()
  {
    this.eventsWindow.close()
  }
  
  Void eventCancel()
  {
    this.eventsWindow.close()
  }
  
  Void eventNew()
  {
    Str? newEventName:=Dialog.openPromptStr(this.eventsWindow, "New Event Name:")
    if ( newEventName != null )
    {
      Int[] previouslySelected:=this.eventsTable.selected
 
      this.diagram.gui.eventRegistry.add(newEventName)
      eventsTable.refreshAll()
      previouslySelected.add(events.size - 1)
      eventsTable.selected=previouslySelected
    }
  }
  EventDef[] events()
  {
    echo(this.diagram.toStr)
    echo(this.diagram.gui.toStr)
    echo(this.diagram.gui.eventRegistry.toStr)
    echo(this.diagram.gui.eventRegistry.events.toStr)
    echo(this.diagram.gui.eventRegistry.events.size)
    return(this.diagram.gui.eventRegistry.events)
  }
  
  Void eventEdit()
  {
  }
  
  Void eventDelete()
  {
  }
  
  Void cancelUpdatesTransition()
  {
  }
  
  Void genStateMachine()
  {
    rootState:=this.diagram.getRootState
    echo("Generating state machine for $this.diagram.getRootState.name") 
    JsmGenerator.generate(this.diagram,this.diagram.gui.eventRegistry,rootState)
  }
  
  Void saveStateMachine()
  {
    this.diagram.saveAction()
    this.saveStateMachineButton.enabled=false
  }
  
  Void editInternalEvents()
  {
  }
  
  Void evAddRegionButtonClick()
  {
    echo("Adding region to $currentState.name")
    JsmRegion? newRegion:=this.currentState.addRegion()
    echo("$currentState.name now has $this.currentState.regions.size regions")
    if ( newRegion != null )
    {
      this.diagram.redrawReason="Added new region"
    }
    this.diagram.checkRedraw()
  }
  
  Void delRegion()
  {
    echo("Deleting region to $currentState.name")
  }
  
  Void displayStateAttributes(JsmState activeState)
  {
    this.currentState=activeState
    this.currentNode=activeState
    this.stateName.text=activeState.name
    this.fillColor.text=activeState.fillColor.toStr
    this.coords.text=activeState.coords
    this.regions.text=activeState.regions.size.toStr
    this.nodeCount.text=activeState.getAllChildren.size.toStr
    this.substateCount.text=activeState.getSubstates.size.toStr
    this.region_1.text=activeState.getFirstRegionCoords.toStr
    if ( activeState.parent != null )
    {
      this.regionName.text=activeState.parent.name
    }
    this.entryActivity.enabled=true
    this.exitActivity.enabled=true
    this.regions.enabled=true
    this.substateCount.enabled=true
    this.region_1.enabled=true
  }
  
  Void displayPseudoStateAttributes(JsmPseudoState activeState)
  {
    this.currentState=null
    this.currentNode=activeState
    this.stateName.text=activeState.name
    this.fillColor.text=activeState.fillColor.toStr
    this.coords.text=activeState.coords
    this.nodeCount.text=activeState.getAllChildren.size.toStr
    if ( activeState.parent != null )
    {
      this.regionName.text=activeState.parent.name
    }
    this.entryActivity.enabled=false
    this.exitActivity.enabled=false
    this.regions.enabled=false
    this.substateCount.enabled=false
    this.region_1.enabled=false
  }
  
  Void displayConnAttributes(JsmConnection activeConn)
  {
    this.currentConn=activeConn
    this.currentState=null
    this.currentNode=null
    echo("Current Connection is $activeConn.name")
    this.connName.text=activeConn.name
    this.guard.text=activeConn.guard
    this.eventsList.text=activeConn.event
    this.action.text=activeConn.action
    echo("Current node is null ")
    if ( activeConn.source.type == NodeType.STATE )
    {
      this.eventsButton.enabled=true 
    }
    else
    {
      this.eventsButton.enabled=false 
    }
    
  }
  
  Void applyUpdatesTransition()
  {
  }
  
  JsmState? incRedo()
  {
    JsmState? rootState:=null
    // undo the latest change
    if ( redoInc.size > 1 )
    {
      echo("--------------------------------------------------")
      // take off the redo stack and put back on undo stack
      lastInc.push(redoInc.pop())
      rootState=readLatestState()
    }
    else
    {
      echo("No redo state")
    }
    return(rootState)
  }
  
  JsmState? incUndo()
  {
    JsmState? rootState:=null
    // undo the latest change
    if ( lastInc.size > 1 )
    {
      echo("--------------------------------------------------")
      // take off the undo stack and put on redo stack
      redoInc.push(lastInc.pop())
      rootState=readLatestState()
    }
    else
    {
      echo("No previous state")
    }
    return(rootState)
  }
  
  JsmState readLatestState()
  {
	  JsmState rootState:=lastInc.pop.in.readObj()
	  // the trick here is that once we readObj we cannot unread it
	  // so in order to be able to restore it again we must create
	  // a new buffer
	  lastInc.push(stateToBuf(rootState))
	      
	  if ( lastInc.size == 1)
	  {
	    // now that we are back that the starting point we
	    // need to clone that starting point
	   this.saveStateMachineButton.enabled=false
	        
	  }
	  echo("--- [${lastInc.size}] Restored to previous state $rootState.name ($rootState)")
	  //canvas.restore(rootState)
	  //canvas.repaint()
	  this.currentConn=null
	  this.currentNode=null
	  this.currentState=null
    return(rootState)
  }
  
  Buf stateToBuf(JsmState state)
  {
    Buf buf:=Buf()
    // write object to the buffer 
    buf.out.writeObj(state)
    // change the buffer from write mode to read mode
    buf.flip
    //echo(buf)
    //echo(buf.size)
    // read back in the object from the buffer
    //echo("+++ [$lastInc.size] Saved state $rootState.name children=$rootState.getAllChildren.size")
    return(buf)
  }
  
  Void incSave()
  {
    this.fileSave(JsmUtil.getFileObj2(
      JsmOptions.instance.backupPath,
      diagramName.text+"_"+
       DateTime.now.toLocale("YYYYMMDD_hhmmss.")+
       DateTime.nowUnique().toStr+".txt")
      )
    currentUpdateNo++
    lastInc.push(stateToBuf(this.diagram.getRootState))
	  echo("--- [${lastInc.size}] Saved state $this.diagram.getRootState.name ($this.diagram.getRootState)")
    echo ("~~~~~~~~~~~~~~~~~~~ Clear REDO BUFFER ~~~~~~~~~~~~~~~~~~~~~~~")
    redoInc.clear()
    this.saveStateMachineButton.enabled=true
  }
  
  Void diagramSave()
  {
    path:=this.diagramPath.text
    if ( path[1] == ':' ) // c:/file.txt  -- check for colon in second character
    {
      path=path.replace("\\","/")
    }
    lastSavedUpdateNo=currentUpdateNo
    echo("Saving $path")
    fileSave(Uri("file:///${path}").toFile)
  }
  
  Bool notSaved()
  {
    if ( lastSavedUpdateNo==currentUpdateNo )
    {
      return(false)
    }
    else
    {
      return(true)
    }
  }
  
  Void fileSave(File f)
  {
      echo("Saving: ${f.osPath}")
    //File f:= Uri("file:///${path}").toFile
    JsmState? rootState:=this.diagram.getRootState
    if (rootState!=null)
    {
      // Create a buffer for an object
      echo("*************** Saving state $rootState.name $rootState.getAllChildren.size nodes")
      echo(Uri("c:/jsm/foo.txt").toStr)
      f.open()
      // write a serialized object (list of things)
      f.writeObj(rootState)
      echo("Saved: ${f.osPath}")
    }
  }

  Void darken()
  {
    if (currentNode!=null)
    {
      currentNode.fillColor=currentNode.fillColor.lighter(0.05f)
    }
  }

  Void lighten()
  {
    if (currentNode!=null)
    {
      currentNode.fillColor=currentNode.fillColor.darker(0.05f)
    }
  }
}

**************************************************************************
** EventTableModel
**************************************************************************

class EventTableModel : TableModel
{
  EventDef[]? events
  Str[] headers := ["Name", "Description"]
  override Int numCols() { return 2 }
  override Int numRows() { return events.size }
  override Str header(Int col) { return headers[col] }
  override Halign halign(Int col) { return col == 1 ? Halign.right : Halign.left }
  override Font? font(Int col, Int row) { return col == 2 ? Font {name=Desktop.sysFont.name; size=Desktop.sysFont.size-1} : null }
  override Color? fg(Int col, Int row)  { return col == 2 ? Color("#666") : null }
  override Color? bg(Int col, Int row)  { return col == 2 ? Color("#eee") : null }
  override Str text(Int col, Int row)
  {
    f := events[row]
    switch (col)
    {
      case 0:  return f.name
      case 1:  return f.description
      //case 1:  return f.size?.toLocale("B") ?: ""
      //case 2:  return f.modified.toLocale
      default: return "?"
    }
  }
  override Int sortCompare(Int col, Int row1, Int row2)
  {
    a := events[row1]
    b := events[row2]
    switch (col)
    {
      case 1:  return a.name <=> b.name
      case 2:  return a.description <=> b.description
      default: return super.sortCompare(col, row1, row2)
    }
  }
  override Image? image(Int col, Int row)
  {
    return null
    //if (col != 0) return null
    //return events[row].isDir ? demo.folderIcon : demo.fileIcon
  }
}

