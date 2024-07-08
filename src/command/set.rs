use crate::command::Execute;
use crate::storage::Expiry;
use crate::storage::{contains_key, insert};
use crate::value::bulk_string::BulkString;
use crate::value::nulls::Nulls;
use crate::value::serialize::Serialize;
use crate::value::simple_error::{ErrorType, SimpleError};
use crate::value::simple_string::SimpleString;
use crate::value::Value;
pub struct SetCommand;
#[derive(Debug)]
enum SetCondition {
    NX,
    XX,
}
type Seconds = u64;
type MilliSeconds = u64;
type EpochSeconds = u128;
type EpochMilliSeconds = u128;
#[derive(Debug)]
enum ExpiryTime {
    EX(Seconds),
    PX(MilliSeconds),
    EXAT(EpochSeconds),
    PXAT(EpochMilliSeconds),
}
impl Execute for SetCommand {
    fn execute(self, options: Vec<BulkString>) -> Box<dyn Serialize> {
        let options_count = options.len();
        let mut set_condition: Option<SetCondition> = None;
        let mut expiry_time: Option<ExpiryTime> = None;
        let key = options.get(0).cloned().unwrap().0;
        let value = options.get(1).cloned().unwrap().0;
        match options_count {
            0 | 1 => {
                return Box::new(Value::SimpleError(SimpleError {
                    error_type: ErrorType::try_from("ERR").unwrap(),
                    message: String::from("wrong number of arguments for 'echo' command"),
                }));
            }
            2 => {
                let _ = insert(key, value, Expiry::INFINITE);
                return Box::new(Value::SimpleString(SimpleString(String::from("Ok"))));
            }
            3..=6 => {
                let mut options_iter = options.iter();
                options_iter.next();
                options_iter.next();
                while let Some(bs) = options_iter.next() {
                    match bs {
                        BulkString(x) if x.to_lowercase() == String::from("nx") => {
                            match set_condition {
                                Some(SetCondition::NX) => continue,
                                Some(SetCondition::XX) => {
                                    return Box::new(Value::SimpleError(SimpleError {
                                        error_type: ErrorType::try_from("ERR").unwrap(),
                                        message: String::from("syntax error"),
                                    }));
                                }
                                None => set_condition = Some(SetCondition::NX),
                            }
                        }
                        BulkString(x) if x.to_lowercase() == String::from("xx") => {
                            match set_condition {
                                Some(SetCondition::XX) => continue,
                                Some(SetCondition::NX) => {
                                    return Box::new(Value::SimpleError(SimpleError {
                                        error_type: ErrorType::try_from("ERR").unwrap(),
                                        message: String::from("syntax error"),
                                    }));
                                }
                                None => set_condition = Some(SetCondition::XX),
                            }
                        }
                        BulkString(x) if x.to_lowercase() == String::from("ex") => {
                            if expiry_time.is_some() {
                                return Box::new(Value::SimpleError(SimpleError {
                                    error_type: ErrorType::try_from("ERR").unwrap(),
                                    message: String::from("syntax error"),
                                }));
                            } else {
                                let val = options_iter.next().cloned();
                                if val.is_none() {
                                    return Box::new(Value::SimpleError(SimpleError {
                                        error_type: ErrorType::try_from("ERR").unwrap(),
                                        message: String::from("syntax error"),
                                    }));
                                } else {
                                    if let Ok(time_value) = val.unwrap().0.parse::<u64>() {
                                        expiry_time = Some(ExpiryTime::EX(time_value));
                                    } else {
                                        return Box::new(Value::SimpleError(SimpleError {
                                            error_type: ErrorType::try_from("ERR").unwrap(),
                                            message: String::from(
                                                "value is not an integer or out of range",
                                            ),
                                        }));
                                    }
                                }
                            }
                        }
                        BulkString(x) if x.to_lowercase() == String::from("px") => {
                            if expiry_time.is_some() {
                                return Box::new(Value::SimpleError(SimpleError {
                                    error_type: ErrorType::try_from("ERR").unwrap(),
                                    message: String::from("syntax error"),
                                }));
                            } else {
                                {
                                    let val = options_iter.next().cloned();
                                    if val.is_none() {
                                        return Box::new(Value::SimpleError(SimpleError {
                                            error_type: ErrorType::try_from("ERR").unwrap(),
                                            message: String::from("syntax error"),
                                        }));
                                    } else {
                                        if let Ok(time_value) = val.unwrap().0.parse::<u64>() {
                                            expiry_time = Some(ExpiryTime::PX(time_value));
                                        } else {
                                            return Box::new(Value::SimpleError(SimpleError {
                                                error_type: ErrorType::try_from("ERR").unwrap(),
                                                message: String::from(
                                                    "value is not an integer or out of range",
                                                ),
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                        BulkString(x) if x.to_lowercase() == String::from("exat") => {
                            if expiry_time.is_some() {
                                return Box::new(Value::SimpleError(SimpleError {
                                    error_type: ErrorType::try_from("ERR").unwrap(),
                                    message: String::from("syntax error"),
                                }));
                            } else {
                                {
                                    let val = options_iter.next().cloned();
                                    if val.is_none() {
                                        return Box::new(Value::SimpleError(SimpleError {
                                            error_type: ErrorType::try_from("ERR").unwrap(),
                                            message: String::from("syntax error"),
                                        }));
                                    } else {
                                        if let Ok(time_value) = val.unwrap().0.parse::<u128>() {
                                            expiry_time = Some(ExpiryTime::EXAT(time_value));
                                        } else {
                                            return Box::new(Value::SimpleError(SimpleError {
                                                error_type: ErrorType::try_from("ERR").unwrap(),
                                                message: String::from(
                                                    "value is not an integer or out of range",
                                                ),
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                        BulkString(x) if x.to_lowercase() == String::from("pxat") => {
                            if expiry_time.is_some() {
                                return Box::new(Value::SimpleError(SimpleError {
                                    error_type: ErrorType::try_from("ERR").unwrap(),
                                    message: String::from("syntax error"),
                                }));
                            } else {
                                {
                                    let val = options_iter.next().cloned();
                                    if val.is_none() {
                                        return Box::new(Value::SimpleError(SimpleError {
                                            error_type: ErrorType::try_from("ERR").unwrap(),
                                            message: String::from("syntax error"),
                                        }));
                                    } else {
                                        if let Ok(time_value) = val.unwrap().0.parse::<u128>() {
                                            expiry_time = Some(ExpiryTime::PXAT(time_value));
                                        } else {
                                            return Box::new(Value::SimpleError(SimpleError {
                                                error_type: ErrorType::try_from("ERR").unwrap(),
                                                message: String::from(
                                                    "value is not an integer or out of range",
                                                ),
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            return Box::new(Value::SimpleError(SimpleError {
                                error_type: ErrorType::try_from("ERR").unwrap(),
                                message: String::from("syntax error"),
                            }));
                        }
                    }
                }

                match expiry_time {
                    None => match set_condition {
                        Some(SetCondition::NX) => {
                            if contains_key(&key) {
                                return Box::new(Value::Nulls(Nulls));
                            } else {
                                let _ = insert(key, value, Expiry::INFINITE);
                                return Box::new(Value::SimpleString(SimpleString(String::from(
                                    "Ok",
                                ))));
                            }
                        }
                        Some(SetCondition::XX) => {
                            if !contains_key(&key) {
                                return Box::new(Value::Nulls(Nulls));
                            } else {
                                let _ = insert(key, value, Expiry::INFINITE);
                                return Box::new(Value::SimpleString(SimpleString(String::from(
                                    "Ok",
                                ))));
                            }
                        }
                        None => {
                            let _ = insert(key, value, Expiry::INFINITE);
                            return Box::new(Value::SimpleString(SimpleString(String::from("Ok"))));
                        }
                    },
                    Some(ExpiryTime::EX(ex)) => match set_condition {
                        Some(SetCondition::NX) => {
                            if contains_key(&key) {
                                return Box::new(Value::Nulls(Nulls));
                            } else {
                                let _ = insert(key, value, Expiry::DURATION(ex * 1000));
                                return Box::new(Value::SimpleString(SimpleString(String::from(
                                    "Ok",
                                ))));
                            }
                        }
                        Some(SetCondition::XX) => {
                            if !contains_key(&key) {
                                return Box::new(Value::Nulls(Nulls));
                            } else {
                                let _ = insert(key, value, Expiry::DURATION(ex * 1000));
                                return Box::new(Value::SimpleString(SimpleString(String::from(
                                    "Ok",
                                ))));
                            }
                        }
                        None => {
                            let _ = insert(key, value, Expiry::DURATION(ex * 1000));
                            return Box::new(Value::SimpleString(SimpleString(String::from("Ok"))));
                        }
                    },
                    Some(ExpiryTime::PX(px)) => match set_condition {
                        Some(SetCondition::NX) => {
                            if contains_key(&key) {
                                return Box::new(Value::Nulls(Nulls));
                            } else {
                                let _ = insert(key, value, Expiry::DURATION(px));
                                return Box::new(Value::SimpleString(SimpleString(String::from(
                                    "Ok",
                                ))));
                            }
                        }
                        Some(SetCondition::XX) => {
                            if !contains_key(&key) {
                                return Box::new(Value::Nulls(Nulls));
                            } else {
                                let _ = insert(key, value, Expiry::DURATION(px));
                                return Box::new(Value::SimpleString(SimpleString(String::from(
                                    "Ok",
                                ))));
                            }
                        }
                        None => {
                            let _ = insert(key, value, Expiry::DURATION(px));
                            return Box::new(Value::SimpleString(SimpleString(String::from("Ok"))));
                        }
                    },
                    Some(ExpiryTime::EXAT(exat)) => match set_condition {
                        Some(SetCondition::NX) => {
                            if contains_key(&key) {
                                return Box::new(Value::Nulls(Nulls));
                            } else {
                                let _ = insert(key, value, Expiry::EPOCH(exat * 1000));
                                return Box::new(Value::SimpleString(SimpleString(String::from(
                                    "Ok",
                                ))));
                            }
                        }
                        Some(SetCondition::XX) => {
                            if !contains_key(&key) {
                                return Box::new(Value::Nulls(Nulls));
                            } else {
                                let _ = insert(key, value, Expiry::EPOCH(exat * 1000));
                                return Box::new(Value::SimpleString(SimpleString(String::from(
                                    "Ok",
                                ))));
                            }
                        }
                        None => {
                            let _ = insert(key, value, Expiry::EPOCH(exat * 1000));
                            return Box::new(Value::SimpleString(SimpleString(String::from("Ok"))));
                        }
                    },
                    Some(ExpiryTime::PXAT(pxat)) => match set_condition {
                        Some(SetCondition::NX) => {
                            if contains_key(&key) {
                                return Box::new(Value::Nulls(Nulls));
                            } else {
                                let _ = insert(key, value, Expiry::EPOCH(pxat));
                                return Box::new(Value::SimpleString(SimpleString(String::from(
                                    "Ok",
                                ))));
                            }
                        }
                        Some(SetCondition::XX) => {
                            if !contains_key(&key) {
                                return Box::new(Value::Nulls(Nulls));
                            } else {
                                let _ = insert(key, value, Expiry::EPOCH(pxat));
                                return Box::new(Value::SimpleString(SimpleString(String::from(
                                    "Ok",
                                ))));
                            }
                        }
                        None => {
                            let _ = insert(key, value, Expiry::EPOCH(pxat));
                            return Box::new(Value::SimpleString(SimpleString(String::from("Ok"))));
                        }
                    },
                }
            }
            _ => {
                return Box::new(Value::SimpleError(SimpleError {
                    error_type: ErrorType::try_from("ERR").unwrap(),
                    message: String::from("syntax error"),
                }));
            }
        }
    }
}
