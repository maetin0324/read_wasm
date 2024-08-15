(module
  (memory 1)
  (func $i64_store
    (i32.const 0)
    (i64.const 42)
    (i64.store)
  )
  (export "i64_store" (func $i64_store))
)