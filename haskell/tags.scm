; ADT definitions
(data_type
    name: (_) @name) @definition.class

; Type family definitions
(type_family
    name: (_) @name) @definition.class

; ADT constructor definitions
(data_constructors
    (data_constructor
      (prefix name: (_) @name) @definition.function))

; Record constructor definitions
((record
    constructor: (constructor) @name) @definition.function)

; GADT constructor definitions

(gadt_constructors
    (gadt_constructor
      (constructor) @name) @definition.function)

; Record field definitions
(record
    (field
      (variable) @name) @definition.function)

; Type synonym definitions
(type_synomym
    name: (_) @name) @definition.class

; Function definitions
(function
    name: (variable) @name) @definition.function

(signature
    name: (variable) @name) @definition.function

; Class definitions
(class
    name: (_) @name) @definition.interface

; Module definitions
(header
    module: (module) @name) @definition.module
