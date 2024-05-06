(module
  (type (func (result i64)))
  (type (func (param i64 i64) (result i64)))
  (func $_start (type 1) (param i64 i64) (result i64)
    (local i64)
    local.get 0
    call $one
    i64.add
  )
  (func $one (type 0) (result i64)
    i64.const 12
    i64.const 34
    i64.add
  )
  (export "_start" (func $_start))
  (export "one" (func $one))
)