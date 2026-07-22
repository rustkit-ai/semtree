; OCaml: top-level definitions; binding nodes hold the name (a field for types,
; a positional child for modules/classes).
(compilation_unit (value_definition  (let_binding pattern: (value_name) @name)) @function)
(compilation_unit (type_definition   (type_binding name: (_) @name)) @struct)
(compilation_unit (module_definition (module_binding (module_name) @name)) @module)
(compilation_unit (class_definition  (class_binding (class_name) @name)) @class)
