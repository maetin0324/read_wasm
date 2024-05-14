(module
  (type (func (param i64 i64) (result i64)))
  (func $_start (type 0) (param i64 i64) (result i64)
    (local i64)
    i64.const 1
    block
      block
      i64.const 1
      drop
      br 0
      end
      unreachable
    end
    i64.const 1
    i64.add
  )
  (export "_start" (func $_start))
)