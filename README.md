# これは何
wasmをパースして実行するミニマムランタイム

# 使い方
```
Usage: read-wasm [OPTIONS] <FILENAME>

Arguments:
  <FILENAME>

Options:
  -e, --entry-point <ENTRY_POINT>  [default: _start]
  -h, --help                       Print help
```

引数にwasmファイルを指定すると、そのファイルをパースして様々を出力する
パース結果は`crate::binary::wasm::Wasm`構造体として表現される
実行は`crate::exec::ExecMachine`を使って行う
返り値は`ExecMachine`の`value_stack`の最後の値となる

# 使用例
```sh
rmaeda🌱 ~/workspace/rust/read-wasm on  main [!] is 📦 v0.1.0 via 🦀 v1.76.0
（▰╹◡╹）❯  cat block.wat
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
)%

rmaeda🌱 ~/workspace/rust/read-wasm on  main [!] is 📦 v0.1.0 via 🦀 v1.76.0
（▰╹◡╹）❯  wat2wasm block.wat

rmaeda🌱 ~/workspace/rust/read-wasm on  main [!] is 📦 v0.1.0 via 🦀 v1.76.0
（▰╹◡╹）❯  cargo run -- block.wasm -l 100
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/read-wasm block.wasm -l 100`
section_id: 1, section_size: 12
TypeSection
section_id: 3, section_size: 3
FunctionSection
section_id: 7, section_size: 10
ExportSection
section_id: 10, section_size: 47
CodeSection
size: 8
size: 36
Wasm {
    type_section: Some(
        [
            FuncType {
                param_types: [
                    I64,
                ],
                return_types: [
                    I64,
                ],
            },
            FuncType {
                param_types: [
                    I64,
                    I64,
                ],
                return_types: [
                    I64,
                ],
            },
        ],
    ),
    function_section: Some(
        [
            Func {
                type_idx: 0,
            },
            Func {
                type_idx: 1,
            },
        ],
    ),
    export_section: Some(
        [
            ExportFunc {
                name: "_start",
                desc: Func,
                func_idx: 0,
            },
        ],
    ),
    code_section: Some(
        [
            Code {
                size: 8,
                locals: [],
                instrs: [
                    LocalGet(
                        0,
                    ),
                    I64Const(
                        0,
                    ),
                    Call(
                        1,
                    ),
                ],
            },
            Code {
                size: 36,
                locals: [
                    LocalVar {
                        count: 1,
                        value_type: I64,
                    },
                ],
                instrs: [
                    Block(
                        Block {
                            block_type: Void,
                        },
                    ),
                    LocalGet(
                        1,
                    ),
                    LocalSet(
                        2,
                    ),
                    LocalGet(
                        0,
                    ),
                    I64Eqz,
                    BrIf(
                        0,
                    ),
                    LocalGet(
                        0,
                    ),
                    LocalGet(
                        1,
                    ),
                    I64Add,
                    LocalSet(
                        2,
                    ),
                    LocalGet(
                        0,
                    ),
                    I64Const(
                        1,
                    ),
                    I64Sub,
                    LocalGet(
                        2,
                    ),
                    Call(
                        1,
                    ),
                    LocalSet(
                        2,
                    ),
                    End,
                    LocalGet(
                        2,
                    ),
                ],
            },
        ],
    ),
}
return Some(I64(5050))
```

# ToDo
- [x] TypeSection, FunctionSection, ExportSection, CodeSectionのパース
- [ ] ImportSection, GlobalSection, TableSection, MemorySection等のパース
- [x] I64.const, addなどの基本的な命令の実装
- [x] call命令で関数呼び出し
- [x] label_stackやblock, br, loopなどの命令の実装
- [x] 再帰関数が動くようにする
- [x] 引数validationの実装
- [ ] 型検査器の実装
- [ ] 全体テストの追加
- [x] 今panic!でごまかしているところを適切にエラーハンドリングする
