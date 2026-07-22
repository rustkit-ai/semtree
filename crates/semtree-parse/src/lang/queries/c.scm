; C — a function's name is buried in its declarator chain, so capture the
; declarator (@decl) and resolve the identifier generically. `body:` keeps this
; to definitions, not mere type references.
(function_definition declarator: (_) @decl) @function
(struct_specifier name: (_) @name body: (_)) @struct
(union_specifier  name: (_) @name body: (_)) @struct
(enum_specifier   name: (_) @name body: (_)) @enum
