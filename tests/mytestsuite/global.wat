(module
  (type $t0 (func))
  (type $t1 (func (result i32)))
  (global $g i32 (i32.const 50))

  (func $f0 (type $t0)
    call $f1
  )
  (func $f1 (type $t1)
    (global.get $g)
  )
  (export "_start" (func $f0))
)