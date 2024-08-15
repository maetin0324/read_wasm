(module 
  (type (func (param i64 i64) (result i64)))
  (import "env" "add" (func $add (type 0)))
  (func $hoge (result i64)
    i64.const 1
    i64.const 2
    call $add
  )
  (export "_start" (func $hoge))
)