extern crate serde;
use serde::ser::{Serialize, Serializer, SerializeStruct};

#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate serde_json;

#[cfg(test)]
extern crate serde_derive;

pub enum JTableResponse<'e, T> {
    Ok(Action<T>),
    Err(&'e str)
}

pub enum Action<T> {
    List(Vec<T>),
    Create(T),
    Update,
    Delete
}

impl<T> Action<T> where T: Serialize {
    fn ser<S: SerializeStruct>(&self, s: &mut S) -> Result<(), S::Error> {
        use Action::*;
        match self {
            Update | Delete => {},
            Create(r) => s.serialize_field("Record", r)?,
            List(l) => s.serialize_field("Records", l)?
        }
        Ok(())
    }
}

impl<'e, T> serde::Serialize for JTableResponse<'e, T> where T: Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let mut ser_struct = serializer.serialize_struct("response", 2)?;
        match self {
            JTableResponse::Ok(value) => {
                ser_struct.serialize_field("Result", "OK")?;
                value.ser(&mut ser_struct)?;
            },
            JTableResponse::Err(msg) => {
                ser_struct.serialize_field("Result", "ERROR")?;
                ser_struct.serialize_field("Message", msg)?;
            }
        }
        ser_struct.end()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_derive::{Deserialize, Serialize};
    #[test]
    fn serialize_list() {
        #[derive(Deserialize, Serialize)]
        struct Project {
            name: String,
            version: u32,
            contributors: Vec<String>,
        }

        assert_eq!(
            serde_json::to_value(
                JTableResponse::Ok(
                    Action::List(vec![
                        Project {
                            name: "Rust".to_string(),
                            version: 2018,
                            contributors: vec!["brson".to_string()]
                        }
                    ])
                )
            ).unwrap(),
            json!({
                "Result": "OK",
                "Records": [{
                    "name": "Rust",
                    "version": 2018,
                    "contributors": ["brson"]
                }]
            })
        );

        assert_eq!(
            serde_json::to_value(
                JTableResponse::Err::<Project>("Requested entry violates constraints")
            ).unwrap(),
            json!({"Result": "ERROR", "Message": "Requested entry violates constraints"})
        );

        assert_eq!(
            serde_json::to_value(
                JTableResponse::Ok(
                    Action::Create(Some(Project{name: "jtable".to_string(), version: 1, contributors: vec![]}))
                )
            ).unwrap(),
            json!({"Result": "OK", "Record": {"name": "jtable", "version": 1, "contributors": []}})
        );
    }
}
