use indexmap::IndexMap;
use nova::newtype;
use std::{ops::Deref, path::Path};

#[derive(Debug, Clone)]
pub enum DependencyOp {
    Or(Vec<Dep>),
    And(Vec<DependencyOp>),
    Dep(Dep),
}

#[derive(Debug, Clone)]
pub struct Dep {
    ty: TypeKey,
    name: String,
}

impl Dep {
    fn parse(input: &str, types: &IndexMap<TypeIndex, TypeSpec>) -> Result<Dep, ()> {
        let x = input.split(":").collect::<Vec<_>>();
        if x.len() != 2 {
            // TODO
            return Err(());
        }

        let ty = TypeKey(x[0].to_string());
        if types.values().find(|x| x.key == ty).is_none() {
            // TODO
            return Err(());
        }

        let name = x[1].to_string();

        Ok(Dep { ty, name })
    }
}

#[derive(Debug, Clone)]
pub struct Dependencies(DependencyOp);

impl Dependencies {
    fn parse(
        types: &IndexMap<TypeIndex, TypeSpec>,
        raw: &Vec<&str>,
    ) -> Result<Dependencies, FieldsError> {
        let op = raw
            .iter()
            .map(|x| {
                if x.contains("OR") {
                    let ors = x
                        .split("OR")
                        .map(|x| x.trim())
                        .map(|x| Dep::parse(x, types))
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap();
                    DependencyOp::Or(ors)
                } else {
                    DependencyOp::Dep(Dep::parse(x, types).unwrap())
                }
            })
            .collect::<Vec<_>>();

        Ok(Dependencies(DependencyOp::And(op)))
    }

    fn empty() -> Dependencies {
        Dependencies(DependencyOp::And(vec![]))
    }
}

#[derive(Debug, Clone)]
pub struct Properties(IndexMap<String, PropSpec>);

impl Deref for Properties {
    type Target = IndexMap<String, PropSpec>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Properties {
    fn parse(
        name: &str,
        raw: &toml::map::Map<String, toml::Value>,
    ) -> Result<Properties, FieldsError> {
        let out = raw
            .iter()
            .map(|(k, v)| {
                Ok((
                    k.to_string(),
                    PropSpec::parse(&format!("{}.{}", name, k), v)?,
                ))
            })
            .collect::<Result<IndexMap<_, _>, _>>()?;
        Ok(Properties(out))
    }

    fn empty() -> Self {
        Self(IndexMap::new())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    String,
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    #[cfg(feature = "uuid")]
    Uuid,
}

impl Type {
    fn as_str(&self) -> &'static str {
        match self {
            Type::String => "string",
            Type::Bool => "bool",
            Type::U8 => "u8",
            Type::U16 => "u16",
            Type::U32 => "u32",
            Type::U64 => "u64",
            Type::I8 => "i8",
            Type::I16 => "i16",
            Type::I32 => "i32",
            Type::I64 => "i64",
            #[cfg(feature = "uuid")]
            Type::Uuid => "uuid",
        }
    }
    fn parse(input: &str) -> Option<Type> {
        Some(match input {
            "string" | "str" | "String" => Self::String,
            "bool" | "Bool" | "boolean" => Self::Bool,
            "u8" => Self::U8,
            "u16" => Self::U16,
            "u32" => Self::U32,
            "u64" => Self::U64,
            "i8" => Self::I8,
            "i16" => Self::I16,
            "i32" => Self::I32,
            "i64" => Self::I64,
            #[cfg(feature = "uuid")]
            "uuid" | "Uuid" => Self::Uuid,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(String),
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    #[cfg(feature = "uuid")]
    Uuid(uuid::Uuid),
}

impl Value {
    pub fn default(ty: Type) -> Value {
        match ty {
            Type::String => Value::String(Default::default()),
            Type::Bool => Value::Bool(Default::default()),
            Type::U8 => Value::U8(Default::default()),
            Type::U16 => Value::U16(Default::default()),
            Type::U32 => Value::U32(Default::default()),
            Type::U64 => Value::U64(Default::default()),
            Type::I8 => Value::I8(Default::default()),
            Type::I16 => Value::I16(Default::default()),
            Type::I32 => Value::I32(Default::default()),
            Type::I64 => Value::I64(Default::default()),
            #[cfg(feature = "uuid")]
            Type::Uuid => Value::Uuid(Default::default()),
        }
    }

    pub fn new(ty: Type, val: &toml::Value) -> Option<Value> {
        match ty {
            Type::String => val.as_str().map(|x| x.to_string()).map(Self::String),
            Type::Bool => val.as_bool().map(Self::Bool),
            Type::U8 => val
                .as_integer()
                .and_then(|x| x.try_into().ok())
                .map(Self::U8),
            Type::U16 => val
                .as_integer()
                .and_then(|x| x.try_into().ok())
                .map(Self::U16),
            Type::U32 => val
                .as_integer()
                .and_then(|x| x.try_into().ok())
                .map(Self::U32),
            Type::U64 => val
                .as_integer()
                .and_then(|x| x.try_into().ok())
                .map(Self::U64),
            Type::I8 => val
                .as_integer()
                .and_then(|x| x.try_into().ok())
                .map(Self::I8),
            Type::I16 => val
                .as_integer()
                .and_then(|x| x.try_into().ok())
                .map(Self::I16),
            Type::I32 => val
                .as_integer()
                .and_then(|x| x.try_into().ok())
                .map(Self::I32),
            Type::I64 => val
                .as_integer()
                .and_then(|x| x.try_into().ok())
                .map(Self::I64),
            #[cfg(feature = "uuid")]
            Type::Uuid => val
                .as_str()
                .and_then(|x| uuid::Uuid::parse_str(x).ok())
                .map(Self::Uuid),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PropSpec {
    pub ty: Type,
    pub default: Option<Value>,
}

impl PropSpec {
    fn parse(name: &str, raw: &toml::Value) -> Result<PropSpec, FieldsError> {
        let ty = raw
            .get("type")
            .ok_or_else(|| FieldsError::MissingField(name.to_string(), "type"))?
            .as_str()
            .and_then(Type::parse)
            .ok_or_else(|| FieldsError::InvalidFieldType {
                field: name.to_string(),
                key: "type",
                ty: "Type (string)",
            })?;

        let default = match raw.get("default") {
            Some(v) => Some(Value::new(ty, v).ok_or_else(|| FieldsError::InvalidFieldType {
                field: name.to_string(),
                key: "default",
                ty: ty.as_str(),
            })?),
            None => None,
        };

        Ok(PropSpec { ty, default })
    }
}

#[derive(Debug, Clone)]
pub struct FieldSpec {
    pub description: String,
    pub dependencies: Dependencies,
    pub properties: Properties,
}

#[newtype(new, display)]
pub type TypeKey = String;

#[newtype(new, display)]
pub type TypeIndex = String;

#[newtype(new, display)]
pub type FieldKey = String;

#[derive(Debug, Clone)]
pub struct TypeSpec {
    pub key: TypeKey,
    pub is_single: bool,
}

#[derive(Debug, Clone)]
pub struct Spec {
    pub name: String,
    pub types: IndexMap<TypeIndex, TypeSpec>,
    pub fields: IndexMap<TypeIndex, IndexMap<FieldKey, FieldSpec>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not load file.")]
    Io(#[from] std::io::Error),

    #[error("Could not parse TOML.")]
    InvalidToml(#[from] toml::de::Error),

    #[error("Error parsing [spec] table.")]
    Spec(#[from] SpecError),

    #[error("Error parsing type table.")]
    Fields(#[from] FieldsError),
}

#[derive(Debug, thiserror::Error)]
pub enum SpecError {
    #[error("[spec] section not found.")]
    SpecMissing,

    #[error("[spec] is missing a `{0}` field.")]
    MissingField(&'static str),

    #[error("`{0}` field in [spec] is not of type `{1}`.")]
    InvalidFieldType(&'static str, &'static str),

    #[error("Value for key `{0}` in [spec.types] is not of type `string`.")]
    InvalidTypeValue(String),
}

#[derive(Debug, thiserror::Error)]
pub enum FieldsError {
    #[error("Undefined types were found: {0}")]
    ExcessKeys(String),

    #[error("Defined types are missing sections: {0}")]
    MissingKeys(String),

    #[error("[{0}] is missing a `{1}` field.")]
    MissingField(String, &'static str),

    #[error("`{key}` field in [{field}] is not of type `{ty}`.")]
    InvalidFieldType {
        field: String,
        key: &'static str,
        ty: &'static str,
    },

    #[error("[{0}] section not found or wrong type.")]
    SectionMissing(String),
}

impl FieldSpec {
    fn parse(
        section: String,
        types: &IndexMap<TypeIndex, TypeSpec>,
        raw: &toml::map::Map<String, toml::Value>,
    ) -> Result<FieldSpec, FieldsError> {
        let description = raw
            .get("description")
            .ok_or_else(|| FieldsError::MissingField(section.clone(), "description"))?
            .as_str()
            .ok_or_else(|| FieldsError::InvalidFieldType {
                field: section.clone(),
                key: "description",
                ty: "string",
            })?
            .to_string();

        let raw_dependencies = match raw.get("dependencies") {
            Some(v) => Some(
                v.as_array()
                    .and_then(|x| x.iter().map(|x| x.as_str()).collect::<Option<Vec<_>>>())
                    .ok_or_else(|| FieldsError::InvalidFieldType {
                        field: section.clone(),
                        key: "dependencies",
                        ty: "array<string>",
                    })?,
            ),
            None => None,
        };

        let dependencies = match raw_dependencies {
            Some(v) => Dependencies::parse(types, &v)?,
            None => Dependencies::empty(),
        };

        let raw_properties = match raw.get("properties") {
            Some(v) => Some(v.as_table().ok_or_else(|| FieldsError::InvalidFieldType {
                field: section.clone(),
                key: "properties",
                ty: "table",
            })?),
            None => None,
        };

        let properties = match raw_properties {
            Some(v) => Properties::parse(&format!("{}.properties", &section), v)?,
            None => Properties::empty(),
        };

        Ok(FieldSpec {
            description,
            dependencies,
            properties,
        })
    }
}

impl Spec {
    #[inline]
    pub fn parse_path<P: AsRef<Path>>(path: P) -> Result<Spec, Error> {
        Self::parse_str(&std::fs::read_to_string(path)?)
    }

    #[inline]
    pub fn parse_str(s: &str) -> Result<Spec, Error> {
        let raw: toml::map::Map<String, toml::Value> = toml::from_str(s)?;
        let (name, types) = Self::parse_spec(&raw)?;
        let fields = Self::parse_fields(&raw, &types)?;

        Ok(Spec {
            name,
            types,
            fields,
        })
    }

    fn parse_fields(
        raw: &toml::map::Map<String, toml::Value>,
        types: &IndexMap<TypeIndex, TypeSpec>,
    ) -> Result<IndexMap<TypeIndex, IndexMap<FieldKey, FieldSpec>>, FieldsError> {
        let undefined_types = raw
            .keys()
            .filter(|x| *x != "spec" && !types.contains_key(&TypeIndex(x.to_string())))
            .cloned()
            .collect::<Vec<String>>();

        if !undefined_types.is_empty() {
            let undefined_types = undefined_types.join(", ");
            return Err(FieldsError::ExcessKeys(undefined_types));
        }

        let missing_keys = types
            .keys()
            .filter(|x| &***x != "spec" && !raw.contains_key(&***x))
            .cloned()
            .collect::<Vec<_>>();

        if !missing_keys.is_empty() {
            let missing_keys = missing_keys
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(FieldsError::MissingKeys(missing_keys));
        }

        let mut out_types = IndexMap::new();

        for k in types.keys() {
            let section = raw
                .get(&k.0)
                .and_then(|x| x.as_table())
                .ok_or_else(|| FieldsError::SectionMissing(k.to_string()))?;

            let mut fields = IndexMap::new();

            for (subsection, value) in section.iter() {
                let name = format!("{}.{}", &k.0, subsection);
                let value = value
                    .as_table()
                    .ok_or_else(|| FieldsError::SectionMissing(name.clone()))?;
                let field_spec = FieldSpec::parse(name, types, value)?;
                fields.insert(FieldKey(subsection.to_string()), field_spec);
            }

            out_types.insert(k.clone(), fields);
        }

        Ok(out_types)
    }

    fn parse_spec(
        raw: &toml::map::Map<String, toml::Value>,
    ) -> Result<(String, IndexMap<TypeIndex, TypeSpec>), SpecError> {
        let raw_spec = raw
            .get("spec")
            .and_then(|x| x.as_table())
            .ok_or_else(|| SpecError::SpecMissing)?;

        let name = raw_spec
            .get("name")
            .ok_or_else(|| SpecError::MissingField("name"))?
            .as_str()
            .ok_or_else(|| SpecError::InvalidFieldType("name", "string"))?
            .to_string();

        let types = raw_spec
            .get("types")
            .ok_or_else(|| SpecError::MissingField("types"))?
            .as_table()
            .ok_or_else(|| SpecError::InvalidFieldType("types", "map<string, string>"))?
            .iter()
            .map(|(k, v)| {
                let k = k.to_string();
                let type_spec = match v {
                    toml::Value::String(s) => Ok(TypeSpec {
                        key: TypeKey(s.to_string()),
                        is_single: false,
                    }),
                    toml::Value::Table(t) => Ok(TypeSpec {
                        key: TypeKey(t.get("key").and_then(|x| x.as_str()).unwrap().to_string()),
                        is_single: t.get("single").is_some(),
                    }),
                    _ => Err(SpecError::InvalidTypeValue(k.clone())),
                }?;
                Ok((TypeIndex(k), type_spec))
            })
            .collect::<Result<IndexMap<_, _>, SpecError>>()?;

        Ok((name, types))
    }
}
