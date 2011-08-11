
class JsmGenerator
{
  static Bool generate(JsmDiagram diagram,EventRegistry evReg, JsmState rootState)
  {
    g:=JsmGenerator(diagram,evReg,rootState)
    return(g.generateSM)
  }
  
  EventRegistry evReg
  JsmState rootState
  Bool hasErrors:=false
  Str[] errors:=Str[,]
  //Str[] actions:=Str[,]
  //Str[] guards:=Str[,]
  Str[] funcs:=Str[,]
  Str klass
  Str sm
  Str ind
  Str nl
  JsmDiagram diagram
  
  new make(JsmDiagram diagram,EventRegistry evReg, JsmState rootState)
  {
    this.diagram=diagram
    this.nl=diagram.settings.newLine
    this.ind=diagram.settings.codeIndent
    this.evReg=evReg
    this.rootState=rootState
    this.sm=rootState.name
    this.klass="StateMachine_${sm}"
  }
  
  Bool generateSM()
  {
    errors.clear
    hasErrors=false
    echo("enum ${sm}_EventTypes {")
    evReg.lookup.keys.sort.each 
    {   
      def:=evReg.get(it)
      echo("    $def.name,   // $def.description") 
    }
    echo("};")
    echo("")
    echo("class ${klass} : public StateMachine {")
    echo("")
    echo("${ind}public $klass(string s=\"$sm\"):StateMachine(s) {}")
    echo("")
    echo("${ind}void extend();")
    echo("")
    echo("${ind}void define()")
    echo("${ind}{")
    echo("${ind}${ind}////////////////////////////////////////////////////////////")
    echo("${ind}${ind}// Events Defined in State Machine $rootState.name")
    echo("${ind}${ind}////////////////////////////////////////////////////////////")
    evReg.lookup.keys.sort.each 
    {   
      def:=evReg.get(it)
      echo("${ind}$ind}defineEvent($def.name,\"$def.name\");   // $def.description") 
    }
    
    
    echo("")
    echo("${ind}${ind}////////////////////////////////////////////////////////////")
    echo("${ind}${ind}// States Defined in State Machine $rootState.name")
    echo("${ind}${ind}////////////////////////////////////////////////////////////")
    generateRegionStateVars("      ","",rootState,rootState.firstRegion)
    generateStateInitialTransitions(rootState)
    
    echo("")
    echo("${ind}${ind}////////////////////////////////////////////////////////////")
    echo("${ind}${ind}// Transitions Defined in State Machine $rootState.name")
    echo("${ind}${ind}////////////////////////////////////////////////////////////")
    generateRegionTransitions("      ","",rootState,rootState.firstRegion)
    
    echo("${ind}}")
    echo("")
    
    if ( this.funcs.size > 0 )
    {
	    echo("")
	    echo("${ind}////////////////////////////////////////////////////////////")
	    echo("${ind}// Logic for State Machine transitions")
	    echo("${ind}////////////////////////////////////////////////////////////")
	    this.funcs.each
	    {
	      echo(it)
	    }
    }
    
    return(this.hasErrors)
  }
  
  Void generateRegionTransitions(Str indent,Str ptr,JsmState s,JsmRegion r)
  {
    r.children.each
    {   
      if ( it.type != NodeType.INITIAL )
      {
        generateStateTransitions(indent+"",it)
      }
    }
  }
  
  Void generateStateTransitions(Str indent,JsmNode n)
  {
    // process not else transitions first - no other implied order
    n.sourceConnections.each
    {   
      if (! ( it.guard.trim.compareIgnoreCase("else") == 0 || it.guard.trim.compareIgnoreCase("[else]") == 0 ))
      {
        generateTransition(indent,it)
      }
    }
    // process else transitions last 
    n.sourceConnections.each
    {   
      if ( it.guard.trim.compareIgnoreCase("else") == 0 || it.guard.trim.compareIgnoreCase("[else]")== 0 )
      {
        generateTransition(indent,it)
      }
    }
    if ( n.type === NodeType.STATE)
    {
      s:=(JsmState)n
      s.regions.each 
      {   
        generateRegionTransitions(indent,"",s,it)
      }
    }
  }
  
  Void generateTransition(Str indent,JsmConnection c)
  {
    if ( c.source == null )
    {
      error("Transition defined with null source from $c.name")
    }
    else if ( c.target == null )
    {
      error("Transition defined with null target from $c.name")
    }
    else
    {
      if ( c.event == "" || c.event == "none" || c.event == "JSM_NULL_EVENT")
      {
        
        generateTransitionBlock(indent,c,"JSM_NULL_EVENT")
      }
      else
      {
        c.event.splitLines.each
        {
          generateTransitionBlock(indent,c,it)
        }
      }
    }
  }
  
  Str getCode(Str c)
  {
    Str s:=""
    if ( c.startsWith("<pre>"))
    {
      s=c[4..-1]
    }
    else
    {
      c.trim.splitLines.each {s+=this.ind;s+=this.ind;s+=it;s+=this.nl  }    
    }
    return(s) 
  }
  
  
  Void generateTransitionBlock(Str indent,JsmConnection c,Str ev)
  {
    Str code:=getCode(c.guard)
    funcNameRegex:=Regex("^[0-9a-zA-Z_]*\$")
    if (  ( c.guard.trim != "none" && c.guard.trim != "" )
       || ( c.action.trim != "none" && c.action.trim != "" ) 
       || ( c.internalTx )
      )
    {
      echo("${indent}tx=addTransition(s_${c.source.name},s_${c.target.name},ev);")
      if ( c.internalTx )
      {
        echo("${indent}tx->setInternal();")
      }
      if ( c.guard.trim != "none" && c.guard.trim != "" )
      {
        if ( funcNameRegex.matches(c.guard.trim) )
        {
          echo("${indent}tx->setGuard(${c.guard.trim});")
        }
        else
        {
          echo("${indent}tx->setGuard(s_${c.source.name},s_${c.target.name},guard_${c.name});")
          funcs.add("${ind}//Guard function for transtion from $c.source.name to $c.target.name on event ${ev}${nl}${ind}bool guard_${c.name}(Event* ev)${nl}${ind}{${nl}${code}${nl}${ind}}${nl}")
        }
      }
      if ( c.action.trim != "none" && c.action.trim != "")
      {
        echo("${indent}tx=addTransition(s_${c.source.name},s_${c.target.name},ev);")
        if ( funcNameRegex.matches(c.action.trim) )
        {
          echo("${indent}tx->setAction(${c.action.trim});")
        }
        else
        {
          echo("${indent}tx->setAction(s_${c.source.name},s_${c.target.name},action_${c.name});")
          funcs.add("${ind}//Action function for transtion from $c.source.name to $c.target.name on event ${ev}${nl}${ind}void action_${c.name}(Event* ev)${nl}${ind}{${nl}${c.action}${nl}${ind}}${nl}")
        }
      }
    }
    else
    {
      echo("${indent}addTransition(s_${c.source.name},s_${c.target.name},ev);")
    }
  }
  
  Void generateStateBlock(Str indent,Str ptr,JsmState s,Str ev)
  {
    funcNameRegex:=Regex("^[0-9a-zA-Z_]*\$")
	  echo("${indent}State *s_$s.name=${ptr}addState(\"$s.name\");")
	  if ( s.entryActivity != "none" && s.entryActivity != "" )
	  {
	    if ( funcNameRegex.matches(s.entryActivity) )
	    {
	      echo("${indent}s_${s.name}->setEntry(${s.entryActivity});")
	    }
	    else
	    {
        Str code:=getCode(s.entryActivity)
	      echo("${indent}s_${s.name}->setEntry(entry_${s.name});")
	      funcs.add("${ind}//Entry function for state $s.name${nl}${ind}void entry_${s.name}(Event* ev)${nl}${ind}{${nl}${code}${nl}${ind}}${nl}")
	    }
	  }
	  if ( s.doActivity != "none" && s.doActivity != "")
	  {
	    if ( funcNameRegex.matches(s.doActivity.trim) )
	    {
	      echo("${indent}s_${s.name}->setDoActivity(${s.doActivity.trim});")
	    }
	    else
	    {
        Str code:=getCode(s.doActivity)
	      echo("${indent}s_${s.name}->setDoActivity(do_${s.name});")
	      funcs.add("${ind}//Background 'do' thread function for state $s.name${nl}${ind}void do_${s.name}(Event* ev)${nl}${ind}{${nl}${code}${nl}${ind}}${nl}")
	    }
	  }
	  if ( s.exitActivity != "none" && s.exitActivity != "")
	  {
	    if ( funcNameRegex.matches(s.exitActivity.trim) )
	    {
	      echo("${indent}s_${s.name}->setExit(${s.exitActivity.trim});")
	    }
	    else
	    {
        Str code:=getCode(s.exitActivity)
	      echo("${indent}s_${s.name}->setExit(exit_${s.name});")
	      funcs.add("${ind}//Exit function for state $s.name${nl}${ind}void exit_${s.name}(Event* ev)${nl}${ind}{${nl}${code}${nl}${ind}}${nl}")
	    }
	  }
  }
  
  
  Void generateRegionStateVars(Str indent,Str ptr,JsmState s,JsmRegion r)
  {
    r.children.each 
    {   
      if ( it.type == NodeType.STATE )
      {
        generateStateBlock(indent,ptr,s,"JSM_NULL_EVENT")
        generateStateVars(indent+" ",(JsmState)it)
      }
    }
  }
  
  Void generateStateVars(Str indent,JsmState s)
  {
    if ( s.regions.size > 0 )
    {
      echo("${indent}//Begin States Defined in state $s.name ")
      s.regions.each 
      {   
        regVar:="s_${s.name}"
        // if there is only one region we add substates directly to the
        // parent state not to a region. They then default to the first region
        if ( s.regions.size > 1 )
        {
          regVar="r_${it.name}"
          //echo("${indent} // States Defined in state $s.name region $it.name")
          echo("${indent} Region *${regVar}=s_${s.name}->addRegion(\"${it.name}\");")
        }
        generateRegionStateVars(indent+"  ",regVar+"->",s,(JsmRegion)it)
        if ( s.regions.size > 1 )
        {
          //echo("${indent} // End States Defined in state $s.name region $it.name")
        }
      }
      echo("${indent}// End States Defined in state $s.name")
    }
  }
  
  
  Void generateStateInitialTransitions(JsmState s)
  {
    s.regions.each 
    {   
      generateRegionInitialTransitions(it)
    }
  }
  
  Void generateRegionInitialTransitions(JsmRegion r)
  {
    r.children.each 
    {   
      if ( it.type == NodeType.STATE )
      {
        generateStateInitialTransitions((JsmState)it)
      }
    }
    generateInitialTransition(r)
  }
  
  Void generateInitialTransition(JsmRegion r)
  {
    r.children.each 
    {   
      if ( it.type == NodeType.INITIAL )
      {
        generateInitialState((JsmInitial)it)
      }
    }
  }
  
  Void generateInitialState(JsmInitial n)
  {
    if ( n.connections.size == 1 )
    {
      JsmConnection c:=n.connections.first
      if ( c.target == null )
      {
        error("Initial Transition defined to null target from $n.name in region $n.parent.name")
      }
      else
      {
        echo("      setInitialTransition(s_$c.target.name);")
      }
    }
    else if ( n.connections.size == 0 )
    {
      error("No Initial Transition defined for $n.name in region $n.parent.name")
    }
    else
    {
      error("Multiple Transitions defined from Initial state for $n.name in region $n.parent.name")
    }
  }
  
  
  Void error(Str errMsg)
  {
    hasErrors=true
    errors.add(errMsg)
    echo("//ERROR: "+errMsg)
  }
}
