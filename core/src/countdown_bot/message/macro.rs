#[macro_export]
macro_rules! impl_cq_tostring {
    ($st:ty,$cq_code:ident) => {
        impl ToString for $st {
            fn to_string(&self) -> String {
                let jsonval = serde_json::to_value(self).unwrap();
                use serde_json::Value;
                let map = match jsonval {
                    Value::Object(map) => map,
                    _ => todo!(),
                };
                let mut out = String::from("[CQ:");
                out.push_str(stringify!($cq_code));
                for (k, v) in map.iter() {
                    match v {
                        Value::Null => continue,
                        t => {
                            out.push(',');
                            out.push_str(k);
                            out.push('=');
                            out.push_str(
                                match t {
                                    Value::Null => todo!(),
                                    Value::Bool(v) => v.to_string(),
                                    Value::Number(v) => v.to_string(),
                                    Value::String(v) => v.to_string(),
                                    Value::Array(_) => todo!(),
                                    Value::Object(_) => todo!(),
                                }
                                .as_str(),
                            );
                        }
                    };
                }
                out.push(']');
                return out;
            }
        }
    };
}