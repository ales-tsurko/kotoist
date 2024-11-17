use koto::runtime::{runtime_error, KIterator, KIteratorOutput, KList, KMap, KValue, ValueMap};
use thiserror::Error;

use crate::orchestrator::Scale;

pub(super) trait Stream {
    type Item;

    fn from_koto_value(value: &KValue) -> Result<Self, Error>
    where
        Self: Sized;

    fn try_next(&mut self) -> Result<Option<Self::Item>, Error>;
}

pub(super) struct StreamF64 {
    value: Option<f64>,
    iterator: Option<KIterator>,
}

impl StreamF64 {
    pub(super) fn from_map(map: &KMap, key: &str, default: f64) -> Result<Self, Error> {
        match map.get(key) {
            Some(value) => StreamF64::from_koto_value(&value),
            None => StreamF64::from_koto_value(&KValue::Number(default.into())),
        }
    }
}

impl Stream for StreamF64 {
    type Item = f64;

    fn from_koto_value(value: &KValue) -> Result<Self, Error>
    where
        Self: Sized,
    {
        match value {
            KValue::Number(num) => Ok(Self {
                value: Some(f64::from(num)),
                iterator: None,
            }),
            KValue::Iterator(iterator) => Ok(Self {
                value: None,
                iterator: Some(iterator.clone()),
            }),
            value => Err(Error::ValueType(
                format!("{}", value.type_as_string()),
                "number or iterator".to_string(),
            )),
        }
    }

    fn try_next(&mut self) -> Result<Option<Self::Item>, Error> {
        if let Some(value) = self.value {
            Ok(Some(value))
        } else {
            // we expect iterator here, if there's no value
            self.iterator
                .as_mut()
                .expect("the iterator is unexpectedly None")
                .next()
                .map(|val| match val {
                    KIteratorOutput::Value(value) => match value {
                        KValue::Number(num) => Ok(f64::from(num)),
                        other => Err(Error::ReturnType(
                            format!("{}", other.type_as_string()),
                            "number".to_string(),
                        )),
                    },
                    KIteratorOutput::ValuePair(_, _) => Err(Error::ReturnType(
                        "value pair".to_string(),
                        "number".to_string(),
                    )),
                    KIteratorOutput::Error(err) => Err(Error::Iterator(format!("{}", err))),
                })
                .transpose()
        }
    }
}

pub(super) struct StreamVecDegree {
    value: Option<Vec<Degree>>,
    iterator: Option<KIterator>,
}

impl StreamVecDegree {
    pub(super) fn from_map(map: &KMap, key: &str, default: f64) -> Result<Self, Error> {
        match map.get(key) {
            Some(value) => Self::from_koto_value(&value),
            None => Self::from_koto_value(&KValue::Number(default.into())),
        }
    }

    fn process_list(&self, list: KList) -> Result<Vec<Degree>, Error> {
        list.data()
            .iter()
            .map(|value| match value {
                KValue::Number(num) => Ok(Degree::Pitch(f64::from(num))),
                KValue::Str(value) => match value.as_str() {
                    "rest" => Ok(Degree::Rest),
                    other => Err(Error::ReturnType(
                        format!("{}", other),
                        "number or rest".to_string(),
                    )),
                },
                other => Err(Error::ReturnType(
                    format!("{}", other.type_as_string()),
                    "number or rest".to_string(),
                )),
            })
            .collect()
    }
}

impl Stream for StreamVecDegree {
    type Item = Vec<Degree>;

    fn from_koto_value(value: &KValue) -> Result<Self, Error>
    where
        Self: Sized,
    {
        match value {
            KValue::Number(num) => Ok(StreamVecDegree {
                value: Some(vec![Degree::Pitch(f64::from(num))]),
                iterator: None,
            }),
            KValue::Str(value) => match value.as_str() {
                "rest" => Ok(StreamVecDegree {
                    value: Some(vec![Degree::Rest]),
                    iterator: None,
                }),
                val => Err(Error::ValueType(
                    format!("{}", val),
                    "number, iterator or rest".to_string(),
                )),
            },
            KValue::Iterator(iterator) => Ok(StreamVecDegree {
                value: None,
                iterator: Some(iterator.clone()),
            }),
            value => Err(Error::ValueType(
                format!("{}", value.type_as_string()),
                "number, iterator or rest".to_string(),
            )),
        }
    }

    fn try_next(&mut self) -> Result<Option<Self::Item>, Error> {
        if let Some(ref value) = self.value {
            return Ok(Some(value.clone()));
        }
        // we expect iterator here, if there's no value
        self.iterator
            .as_mut()
            .expect("the iterator is unexpectedly None")
            .next()
            .map(|val| match val {
                KIteratorOutput::Value(value) => match value {
                    KValue::Number(num) => Ok(vec![Degree::Pitch(f64::from(num))]),
                    KValue::Str(value) => match value.as_str() {
                        "rest" => Ok(vec![Degree::Rest]),
                        other => Err(Error::ReturnType(
                            format!("{}", other),
                            "number or rest".to_string(),
                        )),
                    },
                    KValue::List(list) => self.process_list(list),
                    other => Err(Error::ReturnType(
                        format!("{}", other.type_as_string()),
                        "number or rest".to_string(),
                    )),
                },
                KIteratorOutput::ValuePair(_, _) => Err(Error::ReturnType(
                    "value pair".to_string(),
                    "number or rest".to_string(),
                )),
                KIteratorOutput::Error(err) => Err(Error::Iterator(format!("{}", err))),
            })
            .transpose()
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Degree {
    Pitch(f64),
    Rest,
}

impl StreamScale {
    pub(super) fn from_map(map: &KMap, key: &str, default: &str) -> Result<Self, Error> {
        match map.get(key) {
            Some(value) => StreamScale::from_koto_value(&value),
            None => StreamScale::from_koto_value(&KValue::Str(default.into())),
        }
    }
}

pub(super) struct StreamScale {
    value: Option<Scale>,
    iterator: Option<KIterator>,
}

impl Stream for StreamScale {
    type Item = Scale;

    fn from_koto_value(value: &KValue) -> Result<Self, Error>
    where
        Self: Sized,
    {
        match value {
            KValue::Str(value) => Scale::try_from(value.as_str().to_uppercase().as_str())
                .map(|scale| Self {
                    value: Some(scale),
                    iterator: None,
                })
                .map_err(|e| Error::Other(format!("{}", e))),
            KValue::Iterator(iterator) => Ok(Self {
                value: None,
                iterator: Some(iterator.clone()),
            }),
            value => Err(Error::ValueType(
                format!("{}", value.type_as_string()),
                "number or iterator".to_string(),
            )),
        }
    }

    fn try_next(&mut self) -> Result<Option<Self::Item>, Error> {
        if let Some(value) = self.value {
            Ok(Some(value))
        } else {
            // we expect iterator here, if there's no value
            self.iterator
                .as_mut()
                .expect("the iterator is unexpectedly None")
                .next()
                .map(|val| match val {
                    KIteratorOutput::Value(value) => match value {
                        KValue::Str(value) => {
                            Scale::try_from(value.as_str().to_uppercase().as_str())
                                .map_err(|e| Error::Other(format!("{}", e)))
                        }
                        other => Err(Error::ReturnType(
                            format!("{}", other.type_as_string()),
                            "number".to_string(),
                        )),
                    },
                    KIteratorOutput::ValuePair(_, _) => Err(Error::ReturnType(
                        "value pair".to_string(),
                        "number".to_string(),
                    )),
                    KIteratorOutput::Error(err) => Err(Error::Iterator(format!("{}", err))),
                })
                .transpose()
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Unexpected value type '{0}' in stream, expected type is {1}")]
    ValueType(String, String),
    #[error("Unexpected return type '{0}' in stream, expected type is {1}")]
    ReturnType(String, String),
    #[error("Error processing iterator: '{0}'")]
    Iterator(String),
    #[error("Error in stream: {0}")]
    Other(String),
}
