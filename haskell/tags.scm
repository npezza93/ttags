; ADT definitions
(adt
    name: (type) @name) @definition.class

; Type family definitions
(type_family
    (_
        name: (type) @name)) @definition.class

; ADT constructor definitions
(constructors
    (data_constructor
      (constructor) @name) @definition.function)

(constructors
    (data_constructor_record
      (constructor) @name) @definition.function)

; GADT constructor definitions

(adt
    (gadt_constructor
      (constructor) @name) @definition.function)

; Record field definitions
(record_fields
    (field
      (variable) @name) @definition.function)

; Type alias definitions
(type_alias
    name: (type) @name) @definition.class

; Function definitions
(function
    name: (variable) @name) @definition.function

(signature
    name: (variable) @name) @definition.function

; Class definitions
(class
    (_
      class: (class_name) @name)) @definition.interface

; Module definitions
(_
    module: (module) @name) @definition.module
