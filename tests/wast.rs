#[allow(unused_imports)]
#[cfg(test)]
mod tests {
  use read_wasm::{binary::wasm::Wasm, exec::{exec_machine::ExecMachine, value::Value, wasi::{self, WasiSnapshotPreview1}}};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
  use std::process::Command;
  #[derive(Debug, Serialize, Deserialize)]
  struct TestSuite {
      source_filename: String,
      commands: Vec<Test>,
  }

  #[derive(Debug, Serialize, Deserialize)]
  #[serde(tag = "type")]
  #[serde(rename_all = "snake_case")]
  enum Test {
      Module {
          line: u32,
          filename: String,
      },
      Action {
          line: u32,
          action: Action,
          expected: Vec<Val>,
      },
      AssertReturn {
          line: u32,
          action: Action,
          expected: Vec<Val>,
      },
      AssertTrap {
          line: u32,
          action: Action,
          text: String,
          expected: Vec<Val>,
      },
      AssertInvalid {
          line: u32,
          filename: String,
          text: String,
          module_type: String,
      },
      AssertMalformed {
          line: u32,
          filename: String,
          text: String,
          module_type: String,
      },
      AssertExhaustion {
          line: u32,
          action: Action,
          text: String,
      },
  }

  #[derive(Debug, Serialize, Deserialize)]
  #[serde(tag = "type")]
  #[serde(rename_all = "snake_case")]
  enum Action {
      Invoke { field: String, args: Vec<Val> },
  }

  #[derive(Debug, Serialize, Deserialize)]
  #[serde(tag = "type")]
  #[serde(rename_all = "snake_case")]
  enum Val {
      I32 { value: Option<String> },
      I64 { value: Option<String> },
      F32 { value: Option<String> },
      F64 { value: Option<String> },
      Externref { value: Option<String> },
      Funcref { value: Option<String> },
  }
  impl Into<Value> for Val {
    fn into(self) -> Value {
      match self {
        Val::I32 { value } => Value::I32(value.unwrap().parse::<u32>().unwrap() as i32),
        Val::I64 { value } => Value::I64(value.unwrap().parse::<u64>().unwrap() as i64),
        Val::F32 { value } => {
          let value = value.unwrap();
          if value == "nan:canonical" {
            Value::F32(f32::NAN)
          } else if value == "nan:arithmetic" {
            Value::F32(f32::NAN)
          } else {
              Value::F32(f32::from_bits(value.parse::<u32>().unwrap()))
          }
        }
        Val::F64 { value } => {
          let value = value.unwrap();
          if value == "nan:canonical" {
            Value::F64(f64::NAN)
          } else if value == "nan:arithmetic" {
            Value::F64(f64::NAN)
          } else {
            Value::F64(f64::from_bits(value.parse::<u64>().unwrap()))
          }
        }
        _ => unimplemented!(),
      }
    }
  }

  async fn test_suite(file_path: &str) {
    // let temp_dir = async_tempfile::TempDir::new().await.unwrap();
    Command::new("wast2json")
      .arg(&format!("./tests/testsuite/{file_path}"))
      .arg("-o")
      .arg("./target/tmp/test.json")
      .output()
      .unwrap();
    
    let mut json = String::new();
    tokio::fs::File::open("./target/tmp/test.json").await.unwrap().read_to_string(&mut json).await.unwrap();
    let test_suite: TestSuite = serde_json::from_str(&json).unwrap();
    let tests = test_suite.commands;
    let mut vm = None;
    let mut wasi = WasiSnapshotPreview1::new();
    for test in tests {
      match test {
        Test::Module { filename, .. } => {
          let filename = format!("./target/tmp/{filename}");
          let wasm = Wasm::new(std::fs::File::open(filename).unwrap());
          vm = Some(ExecMachine::init(wasm, "_start", vec![]));
        }
        Test::AssertReturn { line: _, action, expected } => {
          let action = match action {
            Action::Invoke { field, args } => {
              let args = args.into_iter().map(|x| x.into()).collect();
              let field = field.to_owned();
              vm.as_mut().unwrap().invoke(&mut wasi, field, args).await.unwrap();
              vm.as_mut().unwrap().value_stack.clone()
            }
          };
          let expected: Vec<Value> = expected.into_iter().map(|x| x.into()).collect();
          for (action, expected) in action.iter().zip(expected) {
            match action {
              Value::F32(value) if value.is_nan() => {
                assert!(matches!(expected, Value::F32(value) if value.is_nan()));
              }
              Value::F64(value) if value.is_nan() => {
                assert!(matches!(expected, Value::F64(value) if value.is_nan()));
              }
              _ => {
                assert_eq!(action, &expected);
              }
            }
          }
        }
        _ => unimplemented!(),
      }
    }
  }

  #[tokio::test]
  async fn test_i32() {
    test_suite("i32.wast").await;
  }
}