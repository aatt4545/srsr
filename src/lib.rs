use regex::Regex;
use base64::{Engine as _, engine::general_purpose};
use flate2::read::ZlibDecoder;
use std::io::Read;
use serde::{Serialize, Deserialize};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[derive(Serialize, Deserialize, Clone)]
pub struct DeobfuscateResult {
    pub original_code: String,
    pub obfuscation_type: String,
    pub confidence: f64,
    pub execution_time_ms: u128,
    pub transformations_applied: Vec<String>,
    pub detected_language: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeobfuscateOptions {
    pub language: String,
    pub obfuscation_type: Option<String>,
}

pub struct Deobfuscator {
    transformers: Vec<Transformer>,
}

struct Transformer {
    name: String,
    languages: Vec<String>,
    transform: Box<dyn Fn(&str, &str) -> String>,
}

impl Transformer {
    fn new(name: &str, languages: Vec<&str>, transform: Box<dyn Fn(&str, &str) -> String>) -> Self {
        Transformer {
            name: name.to_string(),
            languages: languages.iter().map(|s| s.to_string()).collect(),
            transform,
        }
    }
    
    fn supports(&self, language: &str) -> bool {
        self.languages.contains(&"all".to_string()) || self.languages.contains(&language.to_string())
    }
}

impl Deobfuscator {
    pub fn new() -> Self {
        let mut transformers = Vec::new();
        
        // ============ 汎用トランスフォーマー ============
        
        transformers.push(Transformer::new(
            "base64", vec!["all"],
            Box::new(|code, lang| {
                let patterns: Vec<&str> = match lang {
                    "php" => vec![r#"base64_decode\('([^']+)'\)"#],
                    "python" => vec![r#"base64\.b64decode\('([^']+)'\)"#],
                    "lua" => vec![r#"decode_base64\('([^']+)'\)"#],
                    "ruby" => vec![r#"Base64\.decode64\('([^']+)'\)"#],
                    "perl" => vec![r#"decode_base64\('([^']+)'\)"#],
                    "java" | "kotlin" => vec![r#"Base64\.getDecoder\(\)\.decode\('([^']+)'\)"#],
                    "go" => vec![r#"base64\.StdEncoding\.DecodeString\('([^']+)'\)"#],
                    "rust" => vec![r#"base64::decode\('([^']+)'\)"#],
                    "shell" | "powershell" => vec![
                        r#"echo '([^']+)' \| base64 -d"#,
                        r#"base64 -d <<< '([^']+)'"#
                    ],
                    _ => vec![
                        r#"atob\('([^']+)'\)"#,
                        r#"btoa\('([^']+)'\)"#
                    ],
                };
                
                let mut result = code.to_string();
                for pattern in patterns {
                    let re = Regex::new(pattern).unwrap();
                    result = re.replace_all(&result, |caps: &regex::Captures| {
                        general_purpose::STANDARD.decode(&caps[1])
                            .map(|d| String::from_utf8_lossy(&d).to_string())
                            .unwrap_or_else(|_| caps[0].to_string())
                    }).to_string();
                }
                result
            }),
        ));
        
        transformers.push(Transformer::new(
            "hex_escape", vec!["all"],
            Box::new(|code, _| {
                let re = Regex::new(r"\\x([0-9a-fA-F]{2})").unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    u8::from_str_radix(&caps[1], 16)
                        .map(|b| (b as char).to_string())
                        .unwrap_or_else(|_| caps[0].to_string())
                }).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "unicode_escape", vec!["all"],
            Box::new(|code, _| {
                let re = Regex::new(r"\\u([0-9a-fA-F]{4})").unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    u32::from_str_radix(&caps[1], 16)
                        .ok()
                        .and_then(char::from_u32)
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| caps[0].to_string())
                }).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "octal_escape", vec!["all"],
            Box::new(|code, _| {
                let re = Regex::new(r"\\([0-7]{3})").unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    u8::from_str_radix(&caps[1], 8)
                        .map(|b| (b as char).to_string())
                        .unwrap_or_else(|_| caps[0].to_string())
                }).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "urlencode", vec!["all"],
            Box::new(|code, _| {
                urlencoding::decode(code).unwrap_or_else(|_| code.to_string().into()).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "rot13", vec!["all"],
            Box::new(|code, _| {
                code.chars().map(|c| {
                    if c.is_ascii_alphabetic() {
                        let base = if c.is_ascii_uppercase() { b'A' } else { b'a' };
                        ((c as u8 - base + 13) % 26 + base) as char
                    } else { c }
                }).collect()
            }),
        ));
        
        transformers.push(Transformer::new(
            "xor", vec!["all"],
            Box::new(|code, _| {
                let re = Regex::new(r"(\d+)\s*\^\s*(\d+)").unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    let a: u32 = caps[1].parse().unwrap_or(0);
                    let b: u32 = caps[2].parse().unwrap_or(0);
                    (a ^ b).to_string()
                }).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "string_concat", vec!["all"],
            Box::new(|code, lang| {
                let pattern = match lang {
                    "php" => r#""([^"]*)"\s*\.\s*"([^"]*)""#,
                    "lua" => r#""([^"]*)"\s*\.\.\s*"([^"]*)""#,
                    "perl" => r#""([^"]*)"\s*\.\s*"([^"]*)""#,
                    "c" | "cpp" => r#""([^"]*)"\s*"([^"]*)""#,
                    "sql" => r"'([^']*)'\s*\|\|\s*'([^']*)'",
                    _ => r#""([^"]*)"\s*\+\s*"([^"]*)""#,
                };
                let re = Regex::new(pattern).unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    format!("\"{}{}\"", &caps[1], &caps[2])
                }).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "reverse", vec!["all"],
            Box::new(|code, _| {
                let re = Regex::new(r#""([^"]+)"\.split\(''\)\.reverse\(\)\.join\(''\)"#).unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    format!("\"{}\"", caps[1].chars().rev().collect::<String>())
                }).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "dead_code", vec!["all"],
            Box::new(|code, _| {
                let mut result = code.to_string();
                let re_false = Regex::new(r"if\s*\(\s*false\s*\)\s*\{[^}]*\}").unwrap();
                result = re_false.replace_all(&result, "").to_string();
                let re_true = Regex::new(r"if\s*\(\s*true\s*\)\s*\{([^}]*)\}").unwrap();
                result = re_true.replace_all(&result, |caps: &regex::Captures| caps[1].to_string()).to_string();
                result
            }),
        ));
        
        // ============ JavaScript/TypeScript ============
        
        transformers.push(Transformer::new(
            "js_charcode", vec!["javascript", "typescript"],
            Box::new(|code, _| {
                let re = Regex::new(r"String\.fromCharCode\(([^)]+)\)").unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    let chars: String = caps[1].split(',')
                        .filter_map(|s| s.trim().parse::<u32>().ok())
                        .filter_map(char::from_u32)
                        .collect();
                    format!("\"{}\"", chars)
                }).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "js_string_array", vec!["javascript", "typescript"],
            Box::new(|code, _| {
                let mut result = code.to_string();
                let array_re = Regex::new(r"var\s+_\w+\s*=\s*\[([^\]]+)\]").unwrap();
                
                if let Some(caps) = array_re.captures(code) {
                    let items: Vec<String> = caps[1].split(',')
                        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                        .collect();
                    
                    let ref_re = Regex::new(r"_(\w+)\[(\d+)\]").unwrap();
                    result = ref_re.replace_all(code, |rc: &regex::Captures| {
                        rc[2].parse::<usize>()
                            .ok()
                            .filter(|&i| i < items.len())
                            .map(|i| format!("\"{}\"", items[i]))
                            .unwrap_or_else(|| rc[0].to_string())
                    }).to_string();
                }
                result
            }),
        ));
        
        transformers.push(Transformer::new(
            "js_control_flow", vec!["javascript", "typescript"],
            Box::new(|code, _| {
                let re = Regex::new(r"while\s*\(\s*!!\[\]\s*\)\s*\{\s*switch\s*\(([^)]+)\)\s*\{(.*?)\}\s*\}").unwrap();
                re.replace_all(code, |caps: &regex::Captures| caps[2].to_string()).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "js_var_rename", vec!["javascript", "typescript"],
            Box::new(|code, _| {
                let mut result = code.to_string();
                let re = Regex::new(r"_0x[0-9a-fA-F]+").unwrap();
                let mut counter = 0;
                result = re.replace_all(&result, |_caps: &regex::Captures| {
                    counter += 1;
                    format!("var_{}", counter)
                }).to_string();
                result
            }),
        ));
        
        transformers.push(Transformer::new(
            "js_eval", vec!["javascript", "typescript"],
            Box::new(|code, _| {
                let re = Regex::new(r"eval\(([^)]+)\)").unwrap();
                re.replace_all(code, |caps: &regex::Captures| caps[1].to_string()).to_string()
            }),
        ));
        
        // ============ Lua ============
        
        transformers.push(Transformer::new(
            "lua_charcode", vec!["lua"],
            Box::new(|code, _| {
                let re = Regex::new(r"string\.char\(([^)]+)\)").unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    let chars: String = caps[1].split(',')
                        .filter_map(|s| s.trim().parse::<u32>().ok())
                        .filter_map(char::from_u32)
                        .collect();
                    format!("\"{}\"", chars)
                }).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "lua_loadstring", vec!["lua"],
            Box::new(|code, _| {
                let re = Regex::new(r"loadstring\(([^)]+)\)").unwrap();
                re.replace_all(code, |caps: &regex::Captures| caps[1].to_string()).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "lua_string_array", vec!["lua"],
            Box::new(|code, _| {
                let mut result = code.to_string();
                let array_re = Regex::new(r"local\s+_\w+\s*=\s*\{([^\}]+)\}").unwrap();
                
                if let Some(caps) = array_re.captures(code) {
                    let items: Vec<String> = caps[1].split(',')
                        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                        .collect();
                    
                    let ref_re = Regex::new(r"_(\w+)\[(\d+)\]").unwrap();
                    result = ref_re.replace_all(code, |rc: &regex::Captures| {
                        rc[2].parse::<usize>()
                            .ok()
                            .filter(|&i| i < items.len())
                            .map(|i| format!("\"{}\"", items[i]))
                            .unwrap_or_else(|| rc[0].to_string())
                    }).to_string();
                }
                result
            }),
        ));
        
        // ============ Python ============
        
        transformers.push(Transformer::new(
            "py_exec", vec!["python"],
            Box::new(|code, _| {
                let re = Regex::new(r"exec\(([^)]+)\)").unwrap();
                re.replace_all(code, |caps: &regex::Captures| caps[1].to_string()).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "py_eval", vec!["python"],
            Box::new(|code, _| {
                let re = Regex::new(r"eval\(([^)]+)\)").unwrap();
                re.replace_all(code, |caps: &regex::Captures| caps[1].to_string()).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "py_lambda", vec!["python"],
            Box::new(|code, _| {
                let re = Regex::new(r"lambda\s*:\s*(.+)").unwrap();
                re.replace_all(code, |caps: &regex::Captures| caps[1].to_string()).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "py_marshal", vec!["python"],
            Box::new(|code, _| {
                let re = Regex::new(r"marshal\.loads\(([^)]+)\)").unwrap();
                re.replace_all(code, |caps: &regex::Captures| caps[1].to_string()).to_string()
            }),
        ));
        
        // ============ PHP ============
        
        transformers.push(Transformer::new(
            "php_eval", vec!["php"],
            Box::new(|code, _| {
                let re = Regex::new(r"eval\(([^)]+)\)").unwrap();
                re.replace_all(code, |caps: &regex::Captures| caps[1].to_string()).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "php_gzinflate", vec!["php"],
            Box::new(|code, _| {
                let re = Regex::new(r#"gzinflate\(base64_decode\('([^']+)'\)\)"#).unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    if let Ok(decoded) = general_purpose::STANDARD.decode(&caps[1]) {
                        let mut decoder = ZlibDecoder::new(&decoded[..]);
                        let mut result = String::new();
                        if decoder.read_to_string(&mut result).is_ok() {
                            return result;
                        }
                    }
                    caps[0].to_string()
                }).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "php_gzuncompress", vec!["php"],
            Box::new(|code, _| {
                let re = Regex::new(r#"gzuncompress\(base64_decode\('([^']+)'\)\)"#).unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    if let Ok(decoded) = general_purpose::STANDARD.decode(&caps[1]) {
                        let mut decoder = ZlibDecoder::new(&decoded[..]);
                        let mut result = String::new();
                        if decoder.read_to_string(&mut result).is_ok() {
                            return result;
                        }
                    }
                    caps[0].to_string()
                }).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "php_str_rot13", vec!["php"],
            Box::new(|code, _| {
                let re = Regex::new(r#"str_rot13\('([^']+)'\)"#).unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    let decoded: String = caps[1].chars().map(|c| {
                        if c.is_ascii_alphabetic() {
                            let base = if c.is_ascii_uppercase() { b'A' } else { b'a' };
                            ((c as u8 - base + 13) % 26 + base) as char
                        } else { c }
                    }).collect();
                    format!("\"{}\"", decoded)
                }).to_string()
            }),
        ));
        
        // ============ Ruby ============
        
        transformers.push(Transformer::new(
            "ruby_eval", vec!["ruby"],
            Box::new(|code, _| {
                let re = Regex::new(r"eval\(([^)]+)\)").unwrap();
                re.replace_all(code, |caps: &regex::Captures| caps[1].to_string()).to_string()
            }),
        ));
        
        // ============ Shell ============
        
        transformers.push(Transformer::new(
            "shell_eval", vec!["shell", "powershell"],
            Box::new(|code, _| {
                let re = Regex::new(r#"eval\s+'([^']+)'"#).unwrap();
                re.replace_all(code, |caps: &regex::Captures| caps[1].to_string()).to_string()
            }),
        ));
        
        // ============ SQL ============
        
        transformers.push(Transformer::new(
            "sql_hex", vec!["sql"],
            Box::new(|code, _| {
                let re = Regex::new(r"0x([0-9a-fA-F]+)").unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    let hex_str = &caps[1];
                    let bytes: Vec<u8> = (0..hex_str.len())
                        .step_by(2)
                        .filter_map(|i| u8::from_str_radix(&hex_str[i..i+2], 16).ok())
                        .collect();
                    String::from_utf8_lossy(&bytes).to_string()
                }).to_string()
            }),
        ));
        
        transformers.push(Transformer::new(
            "sql_char", vec!["sql"],
            Box::new(|code, _| {
                let re = Regex::new(r"CHAR\(([^)]+)\)").unwrap();
                re.replace_all(code, |caps: &regex::Captures| {
                    let chars: String = caps[1].split(',')
                        .filter_map(|s| s.trim().parse::<u32>().ok())
                        .filter_map(char::from_u32)
                        .collect();
                    format!("'{}'", chars)
                }).to_string()
            }),
        ));
        
        Deobfuscator { transformers }
    }
    
    pub fn deobfuscate_with_options(&self, code: &str, options: &DeobfuscateOptions) -> DeobfuscateResult {
        let start = std::time::Instant::now();
        let mut result_code = code.to_string();
        let mut transformations = Vec::new();
        
        let detected_language = if options.language.is_empty() {
            self.detect_language(code)
        } else {
            options.language.clone()
        };
        
        let detected_type = match &options.obfuscation_type {
            Some(t) if !t.is_empty() => t.clone(),
            _ => self.detect_obfuscation_type(code),
        };
        
        if let Some(specific_type) = &options.obfuscation_type {
            if !specific_type.is_empty() {
                for transformer in &self.transformers {
                    if transformer.name == *specific_type && transformer.supports(&detected_language) {
                        let before = result_code.clone();
                        result_code = (transformer.transform)(&result_code, &detected_language);
                        if before != result_code {
                            transformations.push(transformer.name.clone());
                        }
                    }
                }
            }
        } else {
            for _ in 0..3 {
                let mut changed = false;
                for transformer in &self.transformers {
                    if transformer.supports(&detected_language) {
                        let before = result_code.clone();
                        result_code = (transformer.transform)(&result_code, &detected_language);
                        if before != result_code {
                            transformations.push(transformer.name.clone());
                            changed = true;
                        }
                    }
                }
                if !changed { break; }
            }
        }
        
        let confidence = if transformations.is_empty() {
            0.0
        } else {
            0.5 + (transformations.len() as f64 * 0.1).min(0.45)
        };
        
        DeobfuscateResult {
            original_code: result_code,
            obfuscation_type: detected_type,
            confidence,
            execution_time_ms: start.elapsed().as_millis(),
            transformations_applied: transformations,
            detected_language,
        }
    }
    
    pub fn deobfuscate(&self, code: &str, language: &str) -> DeobfuscateResult {
        self.deobfuscate_with_options(code, &DeobfuscateOptions {
            language: language.to_string(),
            obfuscation_type: None,
        })
    }
    
    fn detect_language(&self, code: &str) -> String {
        if code.contains("<?php") || code.contains("$variable") || code.contains("echo ") {
            return "php".to_string();
        }
        if code.contains("def ") || code.contains("import ") || code.contains("self.") || 
           code.contains("__init__") || code.contains("lambda ") {
            return "python".to_string();
        }
        if code.contains("local ") || code.contains("loadstring") || code.contains("string.char") {
            return "lua".to_string();
        }
        if code.contains("function") || code.contains("var ") || code.contains("const ") || 
           code.contains("=>") || code.contains("document.") || code.contains("window.") ||
           code.contains("console.log") {
            return "javascript".to_string();
        }
        if code.contains("interface ") || code.contains(": string") || code.contains(": number") {
            return "typescript".to_string();
        }
        if code.contains("def ") && code.contains("end") && code.contains("puts ") {
            return "ruby".to_string();
        }
        if code.contains("my $") || code.contains("use strict") || code.contains("sub ") {
            return "perl".to_string();
        }
        if code.contains("#include") && code.contains("std::") || code.contains("cout <<") ||
           code.contains("cin >>") {
            return "cpp".to_string();
        }
        if code.contains("#include") || code.contains("int main") || code.contains("printf(") {
            return "c".to_string();
        }
        if code.contains("using System") || code.contains("namespace ") || code.contains("public class") {
            return "csharp".to_string();
        }
        if code.contains("import java") || code.contains("public static void main") ||
           code.contains("System.out.println") {
            return "java".to_string();
        }
        if code.contains("fun ") || code.contains("val ") || code.contains("println") {
            return "kotlin".to_string();
        }
        if code.contains("import Swift") || code.contains("func ") || code.contains(": String") {
            return "swift".to_string();
        }
        if code.contains("package main") || code.contains("func main()") || code.contains("fmt.Println") {
            return "go".to_string();
        }
        if code.contains("fn main") || code.contains("println!") || code.contains("let mut") ||
           code.contains("use std::") {
            return "rust".to_string();
        }
        if code.contains("#!/bin/bash") || code.contains("#!/bin/sh") || code.contains("if [") {
            return "shell".to_string();
        }
        if code.contains("Write-Host") || code.contains("Get-Command") || code.contains("$env:") {
            return "powershell".to_string();
        }
        if code.contains("SELECT ") || code.contains("INSERT INTO") || code.contains("CREATE TABLE") {
            return "sql".to_string();
        }
        if code.contains("<html") || code.contains("<div") || code.contains("<style") {
            return "html".to_string();
        }
        if code.contains("{") && code.contains("}") && code.contains(":") && code.contains(",") {
            return "json".to_string();
        }
        if code.contains("<?xml") || code.contains("<root>") || code.contains("<item>") {
            return "xml".to_string();
        }
        
        "unknown".to_string()
    }
    
    fn detect_obfuscation_type(&self, code: &str) -> String {
        let checks: Vec<(&str, &str)> = vec![
            ("jsfuck", r"\[\]\+\[\]"),
            ("aaencode", "ωﾟﾉ"),
            ("jjencode", r"\$=~\[\]"),
            ("base64", r"atob\(|base64_decode\(|base64\.b64decode|Base64\.decode64|base64\.StdEncoding"),
            ("string_array", r"var\s+_\w+\s*=\s*\[|local\s+_\w+\s*=\s*\{"),
            ("control_flow", r"while\s*\(\s*!!"),
            ("loadstring", r"loadstring\("),
            ("xor", r"\^\d+"),
            ("gzinflate", r"gzinflate\(|gzuncompress\("),
            ("urlencode", r"%[0-9a-fA-F]{2}"),
            ("hex_escape", r"\\x[0-9a-fA-F]{2}"),
            ("unicode_escape", r"\\u[0-9a-fA-F]{4}"),
            ("charcode", r"String\.fromCharCode|string\.char\("),
            ("rot13", r"str_rot13"),
            ("eval", r"eval\(|exec\(|loadstring\("),
        ];
        
        for (name, pattern) in checks {
            if Regex::new(pattern).unwrap().is_match(code) {
                return name.to_string();
            }
        }
        
        "unknown".to_string()
    }
}

// ============ FFI for Go ============

#[no_mangle]
pub extern "C" fn deobfuscate_code(code: *const c_char, language: *const c_char) -> *mut c_char {
    let code_str = unsafe {
        CStr::from_ptr(code).to_string_lossy().into_owned()
    };
    let lang_str = unsafe {
        CStr::from_ptr(language).to_string_lossy().into_owned()
    };
    
    let deobfuscator = Deobfuscator::new();
    let result = deobfuscator.deobfuscate(&code_str, &lang_str);
    
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
