(module
  (type (func (param i64) (result i64)))
  (type (func (param i64 i64) (result i64)))
  (func $_start (type 0) (param i64) (result i64)
    local.get 0
    i64.const 0
    call 1
  )
  (func $_arithmetic_seriese (type 1) (param i64 i64) (result i64)
    (local i64)
    block
      local.get 1
      local.set 2
      local.get 0
      i64.eqz
      br_if 0
      local.get 0
      local.get 1
      i64.add
      local.set 2
      local.get 0
      i64.const 1
      i64.sub
      local.get 2
      call 1
      local.set 2
    end
    local.get 2
  )
  (export "_start" (func $_start))
)