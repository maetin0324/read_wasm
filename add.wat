(module
  (type (func (param i64) (result i64)))
  (type (func (result i64)))
  (func $_start (type 1) (result i64)
    i64.const 1
    i64.const 99999999
    i64.add
  )
  (func $one (type 0) (param i64) (result i64)
    (local i64)
    (local i32)
    i64.const 1
  )
  (export "_start" (func $_start))
  (export "one" (func $one))
)