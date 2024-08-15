(module 
  (type (func (result i64)))
  (type (func ))
  (import "env" "one" (func $one (type 0)))
  (import "env" "none" (func $none (type 1)))
  (func $hoge (type 0) 
    i64.const 1
    i64.const 2
    i64.add
  )
  (export "_start" (func $hoge))
)