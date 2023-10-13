; Functions
(binding 
  attrpath: (attrpath attr: (identifier))
  expression: (function_expression)) @definition.function

; Attrset bindings
(binding
  attrpath: (attrpath attr: (identifier))
  expression: (apply_expression)) @definition.class
