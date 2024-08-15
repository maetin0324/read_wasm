(module 
  (type (func (result i64)))
  (func (type 0) (result i64)
    i64.const 1
    i64.const 2
    i64.add
  )
  (export "_start" (func 0))
)