grammar AgentLang

entry Model:
    'agent' id=ID '{'
        'name' name=STRING
        'version' version=VERSION
        'type' type=ID
        ('description' description=STRING)?
        identity=Identity
        objective=Objective
        capabilities+=Capability+
        actions+=Action+
        policy=Policy
        memory=Memory
        evaluation=Evaluation
    '}';

Identity:
    'identity' '{'
        'owner' owner=ID
        'tenant' tenant=ID
        ('issuer' issuer=ID)?
        ('subject' subject=ID)?
        'lifecycle' lifecycle=ID
    '}';

Objective:
    'objective' primary=STRING '{'
        ('success' success+=STRING)*
        ('constraint' constraints+=STRING)*
    '}';

Capability:
    'capability' id=ID '{'
        'name' name=STRING
        'risk' risk=ID
        ('description' description=STRING)?
    '}';

Action:
    'action' id=ID '{'
        'name' name=STRING
        'target' target=STRING
        'reversible' reversible=BOOL
        'approval' approval=ID
        'policy' policy=ID
    '}';

Policy:
    'policy' '{'
        'deny-by-default' denyByDefault=BOOL
        ('ref' refs+=STRING)*
        ('engine' engine=STRING)?
    '}';

Memory:
    'memory' '{'
        'enable' enable+=('working' | 'episodic' | 'semantic' | 'procedural')+
        'retention' retention=STRING
    '}';

Evaluation:
    'evaluation' '{'
        ('metric' metrics+=STRING)*
        ('test' test=STRING)?
        'min-trust' minTrust=NUMBER
    '}';

terminal BOOL returns boolean: /true|false/;
terminal VERSION: /[0-9]+\.[0-9]+\.[0-9]+/;
terminal NUMBER returns number: /[0-9]+(\.[0-9]+)?/;
terminal STRING: /"[^"]*"/;
terminal ID: /[a-zA-Z_][a-zA-Z0-9_\-]*/;
hidden terminal WS: /\s+/;
hidden terminal COMMENT: /#[^\n]*/;
