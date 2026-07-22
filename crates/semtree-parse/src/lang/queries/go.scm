; Go — an interface type_spec is a Trait, a struct type_spec a Struct.
(function_declaration name: (_) @name) @function
(method_declaration   name: (_) @name) @method
(type_declaration (type_spec name: (_) @name type: (interface_type))) @trait
(type_declaration (type_spec name: (_) @name type: (struct_type)))    @struct
