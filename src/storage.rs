use crate::error::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
type MilliSeconds = u64;
type EpochMilliSeconds = u128;

#[derive(Debug, Clone)]
pub enum Expiry {
    DURATION(MilliSeconds),
    EPOCH(EpochMilliSeconds),
    INFINITE,
}
#[derive(Debug, Clone)]
struct DBEntry {
    value: String,
    created_time: SystemTime,
    expiry: Expiry,
}
type Database = Arc<Mutex<HashMap<String, DBEntry>>>;

static DB: Lazy<Database> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub fn contains_key(key: &String) -> bool {
    if DB.lock().unwrap().contains_key(key) {
        return true;
    } else {
        return false;
    }
}

pub fn insert(key: String, value: String, expiry: Expiry) -> Result<()> {
    let to_insert = DBEntry {
        value,
        created_time: SystemTime::now(),
        expiry,
    };
    DB.lock().unwrap().insert(key, to_insert);
    Ok(())
}
pub fn get(key: String) -> Result<String> {
    let db = DB
        .lock()
        .expect("Could not lock DB while trying to get the value for a key");
    let db_get_output = db.get(&key);
    if db_get_output.is_none() {
        return Err(RedisError::KeyDoesNotExist);
    }
    let db_entry = db_get_output.unwrap();
    let db_entry_cloned = db_entry.clone();
    let (value, created_time, expiry) = (
        db_entry_cloned.value,
        db_entry_cloned.created_time,
        db_entry_cloned.expiry,
    );
    drop(db);
    match expiry {
        Expiry::DURATION(ms) => {
            if created_time.elapsed().unwrap() > Duration::from_millis(ms) {
                let _ = delete(vec![key.to_owned()]);
                Err(RedisError::ExpiredKey)
            } else {
                Ok(value.to_owned())
            }
        }
        Expiry::EPOCH(epoch_ms) => {
            let current_epoch_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();
            if current_epoch_ms > epoch_ms {
                delete(vec![key.to_owned()]);
                Err(RedisError::ExpiredKey)
            } else {
                Ok(value.to_owned())
            }
        }
        Expiry::INFINITE => Ok(value.to_owned()),
    }
}
pub fn delete(keys: Vec<String>) -> usize {
    let mut deleted_keys_count = 0;
    for key in keys {
        if let Some(_) = DB.lock().expect("Could not lock").remove(&key) {
            deleted_keys_count += 1;
        }
    }
    deleted_keys_count
}
