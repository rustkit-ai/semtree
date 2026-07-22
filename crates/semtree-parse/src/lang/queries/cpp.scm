; C++ — like C, plus classes and namespaces.
(function_definition declarator: (_) @decl) @function
(class_specifier  name: (_) @name body: (_)) @class
(struct_specifier name: (_) @name body: (_)) @struct
(union_specifier  name: (_) @name body: (_)) @struct
(enum_specifier   name: (_) @name body: (_)) @enum
(namespace_definition name: (_) @name) @module
