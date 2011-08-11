
class JsmEvent
{
}


**************************************************************************
** EventDef
**************************************************************************
@Serializable
class EventDef
{
  Str name
  Str description
  new maker(Str name,Str description)
  {
    this.name=name 
    this.description=description 
  }
  
  new make(|This| f) { f(this) }
  
}

@Serializable
class EventRegistry
{
  @Transient Bool changed:=false
  @Transient File? file
  @Transient EventDef[]? events
  Str:EventDef lookup := Str:EventDef[:] //HashMap
  new maker() 
  { 
    events:=EventDef[,]
    echo("created new events hash")
    echo("events ${events.size}")
  }
  
  
  new make(|This| f) 
  { 
    f(this) 
    events=lookup.vals
  }
  
  Void saveChanges()
  {
    if ( changed == true )
    {
      this.file.open
      this.file.writeObj(this)
      this.changed=false
      echo("[info] Saved changes to disk for $file.osPath")
    }
    else
    {
      echo("[info] No changes to event registry")
    }
  }
  
  
  Void add(Str newEventName)
  {
    if ( get(newEventName) != null )
    {
      echo("[error] $newEventName already exists in event registry")
    }
    else
    {
      evDef:=EventDef.maker(newEventName,"")
      events.add(evDef) 
      lookup.add(newEventName, evDef)
      changed=true
    }
  }
  
  EventDef? get(Str name)
  {
    lookup.get(name)
  }
}
