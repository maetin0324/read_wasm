(module
  (type (;0;) (func (result i32)))
  (type (;1;) (func (result i64)))
  (type (;2;) (func (result f32)))
  (type (;3;) (func (result f64)))
  (type (;4;) (func (param i32) (result i32)))
  (type (;5;) (func (param i64) (result i64)))
  (type (;6;) (func (param f32) (result f32)))
  (type (;7;) (func (param f64) (result f64)))
  (type (;8;) (func (param i64 f32 f64 i32 i32)))
  (type (;9;) (func (param i64 f32 f64 i32 i32) (result f64)))
  (func (;0;) (type 0) (result i32)
    (local i32)
    local.get 0)
  (func (;1;) (type 1) (result i64)
    (local i64)
    local.get 0)
  (func (;2;) (type 2) (result f32)
    (local f32)
    local.get 0)
  (func (;3;) (type 3) (result f64)
    (local f64)
    local.get 0)
  (func (;4;) (type 4) (param i32) (result i32)
    local.get 0)
  (func (;5;) (type 5) (param i64) (result i64)
    local.get 0)
  (func (;6;) (type 6) (param f32) (result f32)
    local.get 0)
  (func (;7;) (type 7) (param f64) (result f64)
    local.get 0)
  (func (;8;) (type 8) (param i64 f32 f64 i32 i32)
    (local f32 i64 i64 f64)
    local.get 0
    i64.eqz
    drop
    local.get 1
    f32.neg
    drop
    local.get 2
    f64.neg
    drop
    local.get 3
    i32.eqz
    drop
    local.get 4
    i32.eqz
    drop
    local.get 5
    f32.neg
    drop
    local.get 6
    i64.eqz
    drop
    local.get 7
    i64.eqz
    drop
    local.get 8
    f64.neg
    drop)
  (func (;9;) (type 9) (param i64 f32 f64 i32 i32) (result f64)
    (local f32 i64 i64 f64)
    f32.const 0x1.6p+2 (;=5.5;)
    local.set 5
    i64.const 6
    local.set 6
    f64.const 0x1p+3 (;=8;)
    local.set 8
    local.get 0
    f64.convert_i64_u
    local.get 1
    f64.promote_f32
    local.get 2
    local.get 3
    f64.convert_i32_u
    local.get 4
    f64.convert_i32_s
    local.get 5
    f64.promote_f32
    local.get 6
    f64.convert_i64_u
    local.get 7
    f64.convert_i64_u
    local.get 8
    f64.add
    f64.add
    f64.add
    f64.add
    f64.add
    f64.add
    f64.add
    f64.add)
  (func (;10;) (type 4) (param i32) (result i32)
    block (result i32)  ;; label = @1
      local.get 0
    end)
  (func (;11;) (type 4) (param i32) (result i32)
    loop (result i32)  ;; label = @1
      local.get 0
    end)
  (func (;12;) (type 4) (param i32) (result i32)
    block (result i32)  ;; label = @1
      local.get 0
      br 0 (;@1;)
    end)
  (func (;13;) (type 4) (param i32) (result i32)
    block (result i32)  ;; label = @1
      local.get 0
      i32.const 1
      br_if 0 (;@1;)
    end)
  (func (;14;) (type 4) (param i32) (result i32)
    block (result i32)  ;; label = @1
      local.get 0
      local.get 0
      br_if 0 (;@1;)
    end)
  (func (;15;) (type 4) (param i32) (result i32)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 0
          br_table 0 (;@3;) 1 (;@2;) 2 (;@1;)
          i32.const 0
          return
        end
        i32.const 1
        return
      end
      i32.const 2
      return
    end
    i32.const 3)
  (func (;16;) (type 4) (param i32) (result i32)
    local.get 0
    return)
  (func (;17;) (type 4) (param i32) (result i32)
    local.get 0
    if (result i32)  ;; label = @1
      local.get 0
    else
      i32.const 0
    end)
  (func (;18;) (type 4) (param i32) (result i32)
    local.get 0
    if (result i32)  ;; label = @1
      i32.const 1
    else
      local.get 0
    end)
  (export "type-local-i32" (func 0))
  (export "type-local-i64" (func 1))
  (export "type-local-f32" (func 2))
  (export "type-local-f64" (func 3))
  (export "type-param-i32" (func 4))
  (export "type-param-i64" (func 5))
  (export "type-param-f32" (func 6))
  (export "type-param-f64" (func 7))
  (export "type-mixed" (func 8))
  (export "read" (func 9))
  (export "as-block-value" (func 10))
  (export "as-loop-value" (func 11))
  (export "as-br-value" (func 12))
  (export "as-br_if-value" (func 13))
  (export "as-br_if-value-cond" (func 14))
  (export "as-br_table-value" (func 15))
  (export "as-return-value" (func 16))
  (export "as-if-then" (func 17))
  (export "as-if-else" (func 18)))
