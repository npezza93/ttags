; Functions
(binding 
  attrpath: (attrpath attr: (identifier) @name)
  expression: (function_expression)) @definition.function

; Attrset bindings
(binding
  attrpath: (attrpath attr: (identifier) @name)
  expression: (apply_expression)) @definition.class
