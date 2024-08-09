# これは何
wasmをパースして実行するミニマムランタイム

# 使い方
```
Usage: read-wasm <COMMAND>

Commands:
  run
  vm
  serialize
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

Usage: read-wasm run [OPTIONS] <FILENAME>

Arguments:
  <FILENAME>

Options:
  -e, --entry-point <ENTRY_POINT>  [default: _start]
  -l, --locals <LOCALS>
  -h, --help                       Print help
```

引数にwasmファイルを指定すると、そのファイルをパースして様々を出力する
パース結果は`crate::binary::wasm::Wasm`構造体として表現される
実行は`crate::exec::ExecMachine`を使って行う
返り値は`ExecMachine`の`value_stack`の最後の値となる

# 使用例
## 再帰関数
与えられた数nに対して、1からnまでの和を計算する関数をwatで書き、それを実行する例
```sh
rmaeda🌱 ~/workspace/rust/read-wasm on  main [$!?] via 🐋 desktop-linuxis 📦 v0.1.0 via 🦀 v1.80.0
（▰╹◡╹）❯  cat tests/testsuite/block.wat
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

rmaeda🌱 ~/workspace/rust/read-wasm on  main [$!?] via 🐋 desktop-linuxis 📦 v0.1.0 via 🦀 v1.80.0
（▰╹◡╹）❯  cargo run -- run -l 100 block.wasm
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/read-wasm run -l 100 block.wasm`
return Some(I64(5050))
```

## Hello World!
```sh
rmaeda🌱 ~/workspace/rust/read-wasm on  main [$!?] via 🐋 desktop-linuxis 📦 v0.1.0 via 🦀 v1.80.0
（▰╹◡╹）❯  cat hello_world.wat
(module
  (import "wasi_snapshot_preview1" "fd_write"
    (func $fd_write (param i32 i32 i32 i32) (result i32))
  )
  (memory 1)
  (data (i32.const 0) "Hello, World!\n")

  (func $hello_world (result i32)
    (local $iovs i32)

    (i32.store (i32.const 16) (i32.const 0))
    (i32.store (i32.const 20) (i32.const 14))

    (local.set $iovs (i32.const 16))

    (call $fd_write
      (i32.const 1)
      (local.get $iovs)
      (i32.const 1)
      (i32.const 24)
    )
  )
  (export "_start" (func $hello_world))
)%

rmaeda🌱 ~/workspace/rust/read-wasm on  main [$!?] via 🐋 desktop-linuxis 📦 v0.1.0 via 🦀 v1.80.0
（▰╹◡╹）❯  cargo run -- run hello_world.wasm
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.11s
     Running `target/debug/read-wasm run hello_world.wasm`
Hello, World!
```

# ToDo
- [x] TypeSection, FunctionSection, ExportSection, CodeSectionのパース
- [ ] ImportSection, GlobalSection, TableSection, MemorySection等のパース
- [x] I64.const, addなどの基本的な命令の実装
- [x] call命令で関数呼び出し
- [x] label_stackやblock, br, loopなどの命令の実装
- [x] 再帰関数が動くようにする
- [x] 引数validationの実装
- [x] `fd_write`の実装
- [ ] 型検査器の実装
- [x] 全体テストの追加
- [x] 今panic!でごまかしているところを適切にエラーハンドリングする
