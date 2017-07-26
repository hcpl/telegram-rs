use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use error;
use parser::{Schema, Parameter};

struct Constructor {
    id: i32,
    name: String,
    params: Vec<Parameter>,
}

#[derive(Default)]
struct Type {
    constructors: Vec<Constructor>,
}

#[derive(Default)]
struct Module {
    types: HashMap<String, Type>,
}

fn translate_typename(
    typename: &str,
    current_module: &Option<String>,
    predicates: &HashMap<String, String>,
) -> String {
    if typename == "!X" {
        "Box<::std::any::Any>".into()
    } else if typename.contains('%') {
        // FIXME: Implement bare types properly!
        translate_typename(&typename.replace('%', ""), current_module, predicates)
    } else if typename.contains("Vector<") || typename.contains("vector<") {
        let s = typename.split(|c| c == '<' || c == '>').collect::<Vec<_>>();
        let typename = translate_typename(s[1], current_module, predicates);

        format!("Vec<{}>", typename)
    } else if typename.contains('.') {
        let s = typename.splitn(2, '.').collect::<Vec<_>>();

        if let Some(ref current_module) = *current_module {
            if current_module == s[0] {
                return s[1].to_string();
            }
        }

        if current_module.is_some() {
            format!("super::{}::{}", s[0], s[1])
        } else {
            format!("self::{}::{}", s[0], s[1])
        }
    } else if predicates.contains_key(typename) {
        predicates[typename].clone()
    } else {
        match typename {
            // Primitive conversion
            "string" => "String".to_string(),
            "Bool" => "bool".to_string(),
            "int" => "i32".to_string(),
            "int128" => "i128".to_string(),
            "int256" => "(i128, i128)".to_string(),
            "Vec<int>" => "Vec<i32>".to_string(),
            "Vec<long>" => "Vec<i64>".to_string(),
            "long" => "i64".to_string(),
            "double" => "f64".to_string(),
            "bytes" => "Vec<u8>".to_string(),

            _ => {
                if current_module.is_some() {
                    format!("super::{}", typename)
                } else {
                    typename.into()
                }
            }
        }
    }
}

fn translate_id(id: &str, current_module: &Option<String>) -> String {
    if id.contains('.') {
        let s = id.splitn(2, '.').collect::<Vec<_>>();

        if let Some(ref current_module) = *current_module {
            if current_module == s[0] {
                return s[1].to_string();
            }
        }

        format!("{}::{}", s[0], s[1])
    } else {
        match id {
            // Keyword
            "type" => "type_".to_string(),
            _ => id.to_string(),
        }
    }
}

fn to_constructor(
    id: &str,
    predicate: &str,
    params: &Vec<Parameter>,
    kind: &str,
) -> error::Result<Option<(Option<String>, String, Constructor)>> {
    // Recognized primitive types are ignored when defined
    // and raised to the associated Rust primitive type when requested
    //  - Bool => bool
    //  - True => true (wtf is this; it's not used anywhere in the schema)
    //  - Vector t => Vec<T>
    //  - Null => ? (figure out what to do with this)
    // TODO(@rust): Is there a clean way to check against a constant set ?
    if kind == "Bool" || kind == "True" || kind == "Vector t" || kind == "Null" {
        return Ok(None);
    }

    // Check for exceptions
    if kind == "PeerSettings" {
        // 1 - PeerSettings doesn't seem to exist (along with the associated method) but
        //     its still in the schema with a seemingly illegal definition
        return Ok(None);
    }

    // Split kind into <module>.<name>
    let s = kind.splitn(2, '.').collect::<Vec<_>>();
    let (module, name) = if s.len() == 1 {
        (None, s[0])
    } else {
        (Some(s[0].to_string()), s[1])
    };

    // Translate
    let c = Constructor {
        id: id.parse::<i32>()?,
        name: predicate.to_string(),
        params: params.clone(),
    };

    Ok(Some((module, name.to_string(), c)))
}

/// Generate Rust definitions to the file from the schema
pub fn generate<P: AsRef<Path>>(filename: P, schema: &Schema) -> error::Result<()> {
    let mut modules = HashMap::<Option<String>, Module>::new();
    let mut predicates = HashMap::<String, String>::new();

    // Translate: Constructors
    for constructor in &schema.constructors {
        let (module, name, c) = match to_constructor(
            &constructor.id,
            &constructor.predicate,
            &constructor.params,
            &constructor.kind,
        )? {
            Some(value) => value,
            None => {
                continue;
            }
        };

        // Add a map for predicate -> typename
        predicates
            .entry(c.name.clone())
            .or_insert_with(|| name.to_string());

        // Build up type in module
        let module_ = &mut modules.entry(module).or_insert_with(Default::default);
        let type_ = &mut module_
            .types
            .entry(name.to_string())
            .or_insert_with(Default::default);
        type_.constructors.push(c);
    }

    // Translate: Methods
    for method in &schema.methods {
        let (module, name, c) =
            match to_constructor(&method.id, &method.method, &method.params, &method.method)? {
                Some(value) => value,
                None => {
                    continue;
                }
            };

        // Add a map for predicate -> typename
        predicates
            .entry(c.name.clone())
            .or_insert_with(|| name.to_string());

        // Build up type in module
        let module_ = &mut modules.entry(module).or_insert_with(Default::default);
        let type_ = Type {
            constructors: vec![c],
        };
        module_.types.insert(name.to_string(), type_);
    }

    // Output buffered information
    let mut f = File::create(filename).unwrap();
    for (module_name, module) in &modules {
        if let Some(ref module_name) = *module_name {
            // Open module
            writeln!(f, "pub mod {} {{", module_name)?;
        }

        if module.types.values().any(|type_| {
            type_.constructors.iter().any(|constructor| {
                constructor.params.iter().any(|param| {
                    param.kind.len() >= 3 && &param.kind[..3] == "int" &&
                        param.kind[3..]
                            .parse::<u64>()
                            .ok()
                            .map_or(false, |bitness| bitness >= 128)
                })
            })
        }) {
            writeln!(f, "use extprim::i128::i128;")?;
        }

        for (name, type_) in &module.types {
            writeln!(f, "#[derive(Debug, SerializeIdentifiable)]")?;

            // Open type
            if type_.constructors.len() == 1 {
                // A single constructor is output as a struct
                writeln!(f, "#[id = \"0x{:x}\"]", type_.constructors[0].id)?;

                if type_.constructors[0].params.is_empty() {
                    // A single constructor with no parameters is a unit
                    writeln!(f, "pub struct {};", name)?;
                    continue;
                } else {
                    writeln!(f, "pub struct {} {{", name)?;
                }
            } else {
                writeln!(f, "pub enum {} {{", name)?;
            }

            for constructor in &type_.constructors {
                let constructor_name = translate_id(&constructor.name, module_name);

                if constructor.params.is_empty() {
                    // No parameters
                    writeln!(f, "  #[id = \"0x{:x}\"]", constructor.id)?;
                    writeln!(f, "  {},", constructor_name)?;
                } else {
                    // Open constructor (if more than 1)
                    if type_.constructors.len() > 1 {
                        writeln!(f, "  #[id = \"0x{:x}\"]", constructor.id)?;
                        writeln!(f, "  {} {{", constructor_name)?;
                    }

                    // Write out parameters
                    for param in &constructor.params {
                        write!(f, "    ")?;

                        if type_.constructors.len() == 1 {
                            write!(f, "pub ")?;
                        }

                        writeln!(
                            f,
                            "{}: {},",
                            translate_id(&param.name, module_name),
                            translate_typename(&param.kind, module_name, &predicates)
                        )?;
                    }

                    // Close constructor (if more than 1)
                    if type_.constructors.len() > 1 {
                        writeln!(f, "  }},")?;
                    }
                }
            }

            // Close type
            writeln!(f, "}}")?;
        }

        if module_name.is_some() {
            // Close module
            writeln!(f, "}}")?;
        }
    }

    Ok(())
}
