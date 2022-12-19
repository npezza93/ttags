(call method: (identifier) @delegate_def
      arguments: (
                  argument_list (simple_symbol) @name
                  (pair key: (hash_key_symbol) value: (true))?
                  (pair key: (hash_key_symbol) @to value: (simple_symbol) @receiver)
                  (pair key: (hash_key_symbol) value: (true))?)
      (#eq? @delegate_def "delegate")
      (#eq? @to "to"))
