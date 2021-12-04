use std::{fmt::Display, path::Path};

use crate::spec::{FieldKey, Spec, TypeIndex, TypeKey, Value};
use indexmap::IndexMap;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error reading profile file.")]
    Io(#[from] std::io::Error),

    #[error("Error parsing TOML profile file.")]
    Toml(#[from] toml::de::Error),

    #[error("Either [profile.bins] or [profile.libs] must be provided.")]
    NoBinsOrLibs,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub spec: Spec,
    pub description: String,
    pub bins: Vec<String>,
    pub libs: Vec<String>,
    pub features: Vec<String>,
    pub config: IndexMap<TypeKey, IndexMap<FieldKey, IndexMap<String, Value>>>,
}

impl Profile {
    #[inline]
    pub fn parse_path<P: AsRef<Path>>(spec: &Spec, path: P) -> Result<Profile, Error> {
        Self::parse_str(spec, &std::fs::read_to_string(path)?)
    }

    fn parse_config(
        spec: &Spec,
        ty: TypeKey,
        v: &toml::Value,
        map: &mut IndexMap<FieldKey, IndexMap<String, Value>>,
    ) {
        let (index, _tyspec) = spec.types.iter().find(|(_, x)| x.key == ty).unwrap();
        match v {
            toml::Value::String(s) => {
                let s = FieldKey::new(s.into());
                let _field_spec = spec.fields.get(index).unwrap().get(&s).unwrap();
                map.insert(s, IndexMap::<String, Value>::new());
            }
            toml::Value::Table(_t) => todo!("Table values not supported here yet"),
            _ => panic!("Unsupported value"),
        }
    }

    #[inline]
    pub fn parse_str(spec: &Spec, s: &str) -> Result<Profile, Error> {
        let raw: toml::map::Map<String, toml::Value> = toml::from_str(s)?;

        let bins = raw
            .get("profile")
            .and_then(|x| x.get("bins"))
            .and_then(|x| x.as_array())
            .map(|x| {
                x.iter()
                    .map(|x| x.as_str().unwrap().to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let libs = raw
            .get("profile")
            .and_then(|x| x.get("libs"))
            .and_then(|x| x.as_array())
            .map(|x| {
                x.iter()
                    .map(|x| x.as_str().unwrap().to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        if bins.is_empty() && libs.is_empty() {
            return Err(Error::NoBinsOrLibs);
        }

        let features = raw
            .get("profile")
            .and_then(|x| x.get("features"))
            .and_then(|x| x.as_array())
            .map(|x| {
                x.iter()
                    .map(|x| x.as_str().unwrap().to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let description = raw
            .get("profile")
            .and_then(|x| x.get("description"))
            .and_then(|x| x.as_str())
            .unwrap()
            .to_string();

        let mut config: IndexMap<TypeKey, IndexMap<FieldKey, IndexMap<String, Value>>> =
            IndexMap::new();

        raw.get("config")
            .and_then(|x| x.as_table())
            .unwrap()
            .iter()
            .for_each(|(k, v)| {
                let k = TypeKey::new(k.into());
                let entry = config.entry(k.clone()).or_default();
                Self::parse_config(spec, k, v, entry);
            });

        raw.iter()
            .filter(|(k, _)| *k != "profile" && *k != "config")
            .for_each(|(k, v)| {
                let type_index = TypeIndex::new(k.into());
                let type_key = spec.types.get(&type_index).unwrap().key.clone();

                v.as_table().unwrap().iter().for_each(|(xk, xv)| {
                    let xk = FieldKey::new(xk.into());
                    match xv {
                        toml::Value::Boolean(x) => {
                            let _field_spec =
                                spec.fields.get(&type_index).unwrap().get(&xk).unwrap();
                            if *x {
                                config
                                    .entry(type_key.clone())
                                    .or_default()
                                    .entry(xk.clone())
                                    .or_default();
                            }
                        }
                        toml::Value::Table(t) => {
                            let field_spec =
                                spec.fields.get(&type_index).unwrap().get(&xk).unwrap();

                            let mut props = t
                                .iter()
                                .map(|(k, v)| {
                                    let prop_spec = field_spec.properties.get(k).unwrap();
                                    let v = Value::new(prop_spec.ty, v)
                                        .unwrap_or_else(|| Value::default(prop_spec.ty));
                                    (k.to_string(), v)
                                })
                                .collect::<IndexMap<_, _>>();

                            field_spec.properties.iter().for_each(|(k, v)| {
                                if let Some(default) = v.default.as_ref() {
                                    if !props.contains_key(k) {
                                        props.insert(k.into(), default.clone());
                                    }
                                }
                            });

                            let m = config
                                .entry(type_key.clone())
                                .or_default()
                                .entry(xk.clone())
                                .or_default();
                            *m = props;
                        }
                        _ => panic!("No."),
                    }
                });
            });

        Ok(Profile {
            bins,
            libs,
            features,
            spec: spec.clone(),
            description,
            config,
        })
    }

    pub fn cfg_flags_map(&self) -> IndexMap<String, Value> {
        use heck::SnakeCase;

        let mut out = IndexMap::new();
        for (ty, v) in self.config.iter() {
            let tyspec = self
                .spec
                .types
                .iter()
                .find(|(_, v)| &v.key == ty)
                .unwrap()
                .1;

            for (ahh, brr) in v {
                if tyspec.is_single {
                    out.insert(
                        format!("{}", ty).to_snake_case(),
                        Value::String(ahh.to_string()),
                    );
                } else {
                    out.insert(format!("{}_{}", ty, ahh).to_snake_case(), Value::Bool(true));
                }
                for (prop_key, prop_val) in brr {
                    let cfg_key = format!("{}_{}_{}", ty, ahh, prop_key).to_snake_case();
                    out.insert(cfg_key, prop_val.clone());
                }
            }
        }

        out
    }

    pub fn rustc_cfg_flags(&self) -> Vec<String> {
        let map = self.cfg_flags_map();
        let mut out = vec![];

        for (k, v) in map {
            if matches!(v, Value::Bool(false)) {
                continue;
            }

            out.push("--cfg".into());
            out.push(match v {
                Value::String(x) => format!("'{}={:?}'", k, x),
                Value::Bool(_) => format!("'{}'", k),
                Value::U8(x) => format!("'{}={}'", k, x),
                Value::U16(x) => format!("'{}={}'", k, x),
                Value::U32(x) => format!("'{}={}'", k, x),
                Value::U64(x) => format!("'{}={}'", k, x),
                Value::I8(x) => format!("'{}={}'", k, x),
                Value::I16(x) => format!("'{}={}'", k, x),
                Value::I32(x) => format!("'{}={}'", k, x),
                Value::I64(x) => format!("'{}={}'", k, x),
                #[cfg(feature = "uuid")]
                Value::Uuid(x) => format!("'{}=\"{}\"'", k, x.to_hyphenated_ref().to_string()),
            });
        }
        out
    }

    pub fn cargo_flags(&self) -> Vec<Vec<String>> {
        let mut out = vec![];

        for bin in self.bins.iter() {
            let mut o = vec![];
            if bin.contains("/") {
                let mut chunks = bin.split("/");
                o.push("-p".into());
                o.push(chunks.next().unwrap().to_string());
                o.push("--bin".into());
                o.push(chunks.next().unwrap().to_string());
            } else {
                o.push("--bin".into());
                o.push(bin.to_string());
            }
            if !self.features.is_empty() {
                o.push("--features".into());
                o.push(format!("\"{}\"", self.features.join("\",\"")));
            }
            out.push(o);
        }

        for lib in self.libs.iter() {
            let mut o = vec![];
            o.push("--lib".into());
            o.push(lib.to_string());
            if !self.features.is_empty() {
                o.push("--features".into());
                o.push(format!("\"{}\"", self.features.join("\",\"")));
            }
            out.push(o);
        }

        out
    }
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.description)?;
        f.write_str("\n\n")?;

        let mut add_nl = false;

        if !self.bins.is_empty() {
            f.write_str("Binaries:\n")?;
            for bin in self.bins.iter() {
                f.write_fmt(format_args!("  {}\n", bin))?;
            }
            add_nl = true;
        }

        if !self.libs.is_empty() {
            f.write_str("Libraries:\n")?;
            for lib in self.libs.iter() {
                f.write_fmt(format_args!("  {}\n", lib))?;
            }
            add_nl = true;
        }

        if !self.features.is_empty() {
            f.write_str("Features:\n")?;
            for feature in self.features.iter() {
                f.write_fmt(format_args!("  {}\n", feature))?;
            }
            add_nl = true;
        }

        if add_nl {
            f.write_str("\n")?;
        }

        for (tykey, v) in self.config.iter() {
            for (fk, v) in v {
                f.write_fmt(format_args!("{}.{}: enabled\n", &tykey, &fk))?;
                if !v.is_empty() {
                    for (pk, pv) in v {
                        f.write_fmt(format_args!("  {}.{}.{} = {}\n", &tykey, &fk, pk, pv))?;
                    }
                }
            }
        }

        f.write_str("\n")?;

        f.write_str("Rust compiler flags:\n")?;
        f.write_str("  ")?;
        f.write_str(&self.rustc_cfg_flags().join(" "))?;
        f.write_str("\n\n")?;

        f.write_str("Cargo flags:\n")?;
        for line in self.cargo_flags() {
            f.write_str("  ")?;
            f.write_str(&line.join(" "))?;
            f.write_str("\n")?;
        }

        Ok(())
    }
}
