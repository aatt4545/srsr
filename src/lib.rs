// src/lib.rs

use regex::Regex;
use base64::{Engine as _, engine::general_purpose};
use serde::{Serialize, Deserialize};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[derive(Serialize, Deserialize, Clone)]
pub struct ObfuscateResult {
    pub obfuscated_code: String,
    pub level: i32,
    pub execution_time_ms: u128,
    pub transformations_applied: Vec<String>,
}

pub struct Obfuscator {
    transformers: Vec<Transformer>,
}

struct Transformer {
    name: String,
    transform: Box<dyn Fn(&str) -> String>,
}

impl Transformer {
    fn new(name: &str, transform: Box<dyn Fn(&str) -> String>) -> Self {
        Transformer {
            name: name.to_string(),
            transform,
        }
    }
}

impl Obfuscator {
    pub fn new() -> Self {
        let mut transformers = Vec::new();

        transformers.push(Transformer::new(
            "string_xor",
            Box::new(|code| {
                let re = Regex::new(r#""([^"]+)""#).unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    let s = &caps[1];
                    let key: u8 = 42;
                    let encoded: String = s.bytes().map(|b| format!("\\{}", (b ^ key) as u8)).collect();
                    format!("\"{}\"", encoded)
                }).to_string()
            }),
        ));

        transformers.push(Transformer::new(
            "string_base64",
            Box::new(|code| {
                let re = Regex::new(r#""([^"]+)""#).unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    let s = &caps[1];
                    let encoded = general_purpose::STANDARD.encode(s.as_bytes());
                    format!("atob(\"{}\")", encoded)
                }).to_string()
            }),
        ));

        transformers.push(Transformer::new(
            "string_hex",
            Box::new(|code| {
                let re = Regex::new(r#""([^"]+)""#).unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    let s = &caps[1];
                    let encoded: String = s.bytes().map(|b| format!("\\x{:02x}", b)).collect();
                    format!("\"{}\"", encoded)
                }).to_string()
            }),
        ));

        transformers.push(Transformer::new(
            "var_mangle",
            Box::new(|code| {
                let re = Regex::new(r"\b(var|let|const|local)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
                let mut counter = 0;
                re.replace_all(code, |caps: &regex::Captures| {
                    counter += 1;
                    format!("{} _0x{:x}", &caps[1], counter)
                }).to_string()
            }),
        ));

        transformers.push(Transformer::new(
            "dead_code",
            Box::new(|code| {
                format!("if (false) {{ var _0xdead = 'garbage'; }}\n{}", code)
            }),
        ));

        transformers.push(Transformer::new(
            "control_flow_wrap",
            Box::new(|code| {
                format!("(function() {{\n{}\n}})();", code)
            }),
        ));

        Obfuscator { transformers }
    }

    pub fn obfuscate(&self, code: &str, level: i32) -> ObfuscateResult {
        let start = std::time::Instant::now();
        let mut result_code = code.to_string();
        let mut transformations = Vec::new();

        for transformer in &self.transformers {
            if level >= 1 && transformer.name == "string_base64" {
                result_code = (transformer.transform)(&result_code);
                transformations.push(transformer.name.clone());
            }
            if level >= 2 && transformer.name == "string_hex" {
                result_code = (transformer.transform)(&result_code);
                transformations.push(transformer.name.clone());
            }
            if level >= 3 && transformer.name == "string_xor" {
                result_code = (transformer.transform)(&result_code);
                transformations.push(transformer.name.clone());
            }
            if level >= 4 && transformer.name == "var_mangle" {
                result_code = (transformer.transform)(&result_code);
                transformations.push(transformer.name.clone());
            }
            if level >= 5 && transformer.name == "dead_code" {
                result_code = (transformer.transform)(&result_code);
                transformations.push(transformer.name.clone());
            }
            if level >= 6 && transformer.name == "control_flow_wrap" {
                result_code = (transformer.transform)(&result_code);
                transformations.push(transformer.name.clone());
            }
        }

        ObfuscateResult {
            obfuscated_code: result_code,
            level,
            execution_time_ms: start.elapsed().as_millis(),
            transformations_applied: transformations,
        }
    }
}

#[no_mangle]
pub extern "C" fn obfuscate_code(code: *const c_char, level: i32) -> *mut c_char {
    let code_str = unsafe {
        CStr::from_ptr(code).to_string_lossy().into_owned()
    };
    
    let obfuscator = Obfuscator::new();
    let result = obfuscator.obfuscate(&code_str, level);
    
    let json = serde_json::to_string(&result).unwrap();
    
    CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    unsafe {
        if !ptr.is_null() {
            let _ = CString::from_raw(ptr);
        }
    }
}
