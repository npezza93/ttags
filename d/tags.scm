(module_def (module_declaration (module_fqn) @name)) @definition.module

(struct_declaration (struct) . (identifier) @name) @definition.class
(interface_declaration (interface) . (identifier) @name) @definition.interface
(enum_declaration (enum) . (identifier) @name) @definition.type

(class_declaration (class) . (identifier) @name) @definition.class
(constructor (this) @name) @definition.method
(destructor (this) @name) @definition.method
(postblit (this) @name . (this)) @definition.method

(manifest_declarator . (identifier) @name) @definition.constant

(function_declaration (identifier) @name) @definition.function

(union_declaration (union) . (identifier) @name) @definition.type

(enum_member . [(deprecated_attribute) (at_attribute)]* . (identifier) @name) @definition.constant
(anonymous_enum_member (type) . (identifier) @name) @definition.constant

; Alias assignments declare the left-hand name, never names in the target type,
; template arguments or function parameters.
(alias_initializer . (identifier) @name) @definition.alias

; Legacy `alias Target Name[, Other];` and function-type declarations.
; A type child excludes `alias this = member`, which introduces no name.
(alias_declaration (type) (identifier) @name @definition.alias)
