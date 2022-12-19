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

(setter
  (identifier) @ignore)

; Class definitions

(
  (comment)* @doc
  .
  [
    (class
      name: [
        (constant) @name
        (scope_resolution
          name: (_) @name)
      ]) @definition.class
    (singleton_class
      value: [
        (constant) @name
        (scope_resolution
          name: (_) @name)
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
      (scope_resolution
        name: (_) @name)
    ]) @definition.module
)

(assignment left: (constant) @name @definition.constant
 (#match? @name "^[A-Z\\d_]+$"))

(call method: (identifier) @definition.scope_def
      (argument_list ((simple_symbol) @name) @definition.scope)
      (#eq? @definition.scope_def "scope"))

(call method: (identifier) @definition.has_many_def
      (argument_list ((simple_symbol) @name) @definition.has_many)
      (#eq? @definition.has_many_def "has_many"))

(call method: (identifier) @definition.has_one_def
      (argument_list ((simple_symbol) @name) @definition.has_one)
      (#eq? @definition.has_one_def "has_one|belongs_to"))

(call method: (identifier) @definition.attr_reader_def
      (argument_list ((simple_symbol) @name) @definition.attr_reader)
      (#eq? @definition.attr_reader_def "attr_reader"))

(call method: (identifier) @definition.attr_accessor_def
      (argument_list ((simple_symbol) @name) @definition.attr_accessor)
      (#eq? @definition.attr_accessor_def "attr_accessor"))

(call method: (identifier) @definition.attr_writer_def
      (argument_list ((simple_symbol) @name) @definition.attr_writer)
      (#eq? @definition.attr_writer_def "attr_writer"))

((call method: (identifier) @definition.delegate_def
      (argument_list ((simple_symbol) @name) @definition.delegate)) @doc
      (#eq? @definition.delegate_def "delegate"))
