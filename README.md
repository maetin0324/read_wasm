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
maetin🌱 ~/workspace/rust/read-wasm on  main [!] is 📦 v0.1.0 via 🦀 v1.77.2
（▰╹◡╹）❯  cat add.wat
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
maetin🌱 ~/workspace/rust/read-wasm on  main [!] is 📦 v0.1.0 via 🦀 v1.77.2
（▰╹◡╹）❯  wat2wasm add.wat

maetin🌱 ~/workspace/rust/read-wasm on  main [!] is 📦 v0.1.0 via 🦀 v1.77.2
（▰╹◡╹）❯  cargo run -- add.wasm
  Compiling read-wasm v0.1.0 (/home/maetin/workspace/rust/read-wasm)
  Finished dev [unoptimized + debuginfo] target(s) in 1.23s
    Running `target/debug/read-wasm add.wasm`
section_id: 1, section_size: 11
TypeSection
section_id: 3, section_size: 3
FunctionSection
section_id: 7, section_size: 16
ExportSection
section_id: 10, section_size: 19
CodeSection
Wasm { type_section: Some([FuncType { param_types: [], return_types: [I64] }, FuncType { param_types: [I64, I64], return_types: [I64] }]), function_section: Some([Func { type_idx: 1 }, Func { type_idx: 0 }]), export_section: Some([ExportFunc { name: "_start", desc: Func, func_idx: 0 }, ExportFunc { name: "one", desc: Func, func_idx: 1 }]), code_section: Some([Code { size: 9, locals: [LocalVar { count: 1, value_type: I64 }], instrs: [LocalGet(0), Call(1), I64Add] }, Code { size: 7, locals: [], instrs: [I64Const(12), I64Const(34), I64Add] }]) }
return Some(I64(47))
```

# ToDo
- [x] TypeSection, FunctionSection, ExportSection, CodeSectionのパース
- [ ] ImportSection, GlobalSection, TableSection, MemorySection等のパース
- [x] I64.const, addなどの基本的な命令の実装
- [x] call命令で関数呼び出し
- [ ] label_stackやblock, br, loopなどの命令の実装
- [ ] 引数validationの実装
- [ ] 型検査器の実装
- [ ] 全体テストの追加
- [ ] 今panic!でごまかしているところを適切にエラーハンドリングする
