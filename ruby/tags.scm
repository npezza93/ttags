; Method definitions

(
  (comment)* @doc
  .
  (method
    name: (_) @name) @definition.method
  (#strip! @doc "^#\\s*")
  (#not-eq? @name "initialize")
  (#select-adjacent! @doc @definition.method)
)

(
  (comment)* @doc
  .
  (method
    name: (_) @name) @definition.constructor
  (#strip! @doc "^#\\s*")
  (#eq? @name "initialize")
  (#select-adjacent! @doc @definition.constructor)
)

(
  (comment)* @doc
  .
  (singleton_method
    name: (_) @name) @definition.singleton_method
  (#strip! @doc "^#\\s*")
  (#select-adjacent! @doc @definition.singleton_method)
)

(alias
  name: (_) @name) @definition.method

; Class definitions

(
  (comment)* @doc
  .
  [
    (class
      name: [
        (constant) @name
        (scope_resolution) @name
      ]) @definition.class
    (singleton_class
      value: [
        (constant) @name
        (scope_resolution) @name
      ]) @definition.class
  ]
  (#strip! @doc "^#\\s*")
  (#select-adjacent! @doc @definition.class)
)

; Module definitions

(
  (module
    name: [
      (constant) @name
      (scope_resolution) @name
    ]) @definition.module
)

(assignment left: (constant) @name @definition.constant
 (#match? @name "^[A-Z\\d_]+$"))

((call method: (identifier) @metadata
      (argument_list ((simple_symbol) @name)) @definition.macro) @doc
      (#match? @metadata "^(has_many|has_one|belongs_to|scope|attr_reader|attr_accessor|attr_writer|delegate)$"))
