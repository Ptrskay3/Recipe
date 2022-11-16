#![allow(clippy::new_without_default, clippy::len_without_is_empty)]
// A wrapper over async_session::Session.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use chrono::{DateTime, Utc};
use serde::Serialize;

pub struct Session {
    inner: async_session::Session,
    regenerate: Arc<AtomicBool>,
}

impl Clone for Session {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            regenerate: self.regenerate.clone(),
        }
    }
}

impl Session {
    /// Create a new session. Generates a random id and matching
    /// cookie value. Does not set an expiry by default
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// # fn main() -> async_session::Result { let rt = tokio::runtime::Runtime::new().unwrap();
    /// let handle = rt.handle();
    /// handle.block_on(async {
    /// let session = Session::new();
    /// assert_eq!(None, session.expiry());
    /// assert!(session.into_cookie_value().is_some());
    /// # Ok(()) }) }
    pub fn new() -> Self {
        Self {
            inner: async_session::Session::new(),
            regenerate: Arc::new(AtomicBool::new(false)),
        }
    }

    /// applies a cryptographic hash function on a cookie value
    /// returned by [`Session::into_cookie_value`] to obtain the
    /// session id for that cookie. Returns an error if the cookie
    /// format is not recognized
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// # fn main() -> async_session::Result { let rt = tokio::runtime::Runtime::new().unwrap();
    /// let handle = rt.handle();
    /// handle.block_on(async {
    /// let session = Session::new();
    /// let id = session.id().to_string();
    /// let cookie_value = session.into_cookie_value().unwrap();
    /// assert_eq!(id, Session::id_from_cookie_value(&cookie_value)?);
    /// # Ok(()) }) }
    /// ```
    pub fn id_from_cookie_value(string: &str) -> Result<String, base64::DecodeError> {
        async_session::Session::id_from_cookie_value(string)
    }

    /// mark this session for destruction. the actual session record
    /// is not destroyed until the end of this response cycle.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// # fn main() -> async_session::Result { let rt = tokio::runtime::Runtime::new().unwrap();
    /// let handle = rt.handle();
    /// handle.block_on(async {
    /// let mut session = Session::new();
    /// assert!(!session.is_destroyed());
    /// session.destroy();
    /// assert!(session.is_destroyed());
    /// # Ok(()) }) }
    pub fn destroy(&mut self) {
        self.inner.destroy()
    }

    /// returns true if this session is marked for destruction
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// # fn main() -> async_session::Result { let rt = tokio::runtime::Runtime::new().unwrap();
    /// let handle = rt.handle();
    /// handle.block_on(async {
    /// let mut session = Session::new();
    /// assert!(!session.is_destroyed());
    /// session.destroy();
    /// assert!(session.is_destroyed());
    /// # Ok(()) }) }

    pub fn is_destroyed(&self) -> bool {
        self.inner.is_destroyed()
    }

    /// Gets the session id
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// # fn main() -> async_session::Result { let rt = tokio::runtime::Runtime::new().unwrap();
    /// let handle = rt.handle();
    /// handle.block_on(async {
    /// let session = Session::new();
    /// let id = session.id().to_owned();
    /// let cookie_value = session.into_cookie_value().unwrap();
    /// assert_eq!(id, Session::id_from_cookie_value(&cookie_value)?);
    /// # Ok(()) }) }
    pub fn id(&self) -> &str {
        self.inner.id()
    }

    /// inserts a serializable value into the session hashmap. returns
    /// an error if the serialization was unsuccessful.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use serde::{Serialize, Deserialize};
    /// # use async_session::Session;
    /// #[derive(Serialize, Deserialize)]
    /// struct User {
    ///     name: String,
    ///     legs: u8
    /// }
    /// let mut session = Session::new();
    /// session.insert("user", User { name: "chashu".into(), legs: 4 }).expect("serializable");
    /// assert_eq!(r#"{"name":"chashu","legs":4}"#, session.get_raw("user").unwrap());
    /// ```
    pub fn insert(&mut self, key: &str, value: impl Serialize) -> Result<(), serde_json::Error> {
        self.inner.insert(key, value)
    }

    /// inserts a string into the session hashmap
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// let mut session = Session::new();
    /// session.insert_raw("ten", "10".to_string());
    /// let ten: usize = session.get("ten").unwrap();
    /// assert_eq!(ten, 10);
    /// ```
    pub fn insert_raw(&mut self, key: &str, value: String) {
        self.inner.insert_raw(key, value)
    }

    /// deserializes a type T out of the session hashmap
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// let mut session = Session::new();
    /// session.insert("key", vec![1, 2, 3]);
    /// let numbers: Vec<usize> = session.get("key").unwrap();
    /// assert_eq!(vec![1, 2, 3], numbers);
    /// ```
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.inner.get(key)
    }

    /// returns the String value contained in the session hashmap
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// let mut session = Session::new();
    /// session.insert("key", vec![1, 2, 3]);
    /// assert_eq!("[1,2,3]", session.get_raw("key").unwrap());
    /// ```
    pub fn get_raw(&self, key: &str) -> Option<String> {
        self.inner.get_raw(key)
    }

    /// removes an entry from the session hashmap
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// let mut session = Session::new();
    /// session.insert("key", "value");
    /// session.remove("key");
    /// assert!(session.get_raw("key").is_none());
    /// assert_eq!(session.len(), 0);
    /// ```
    pub fn remove(&mut self, key: &str) {
        self.inner.remove(key)
    }

    /// returns the number of elements in the session hashmap
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// let mut session = Session::new();
    /// assert_eq!(session.len(), 0);
    /// session.insert("key", 0);
    /// assert_eq!(session.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Marks the session to generate a new id and cookie for this session.
    /// 
    /// The action is done in the session middleware __after__ your handler is called.
    pub fn regenerate(&mut self) {
        self.regenerate.store(true, Ordering::Relaxed);
    }

    /// sets the cookie value that this session will use to serialize
    /// itself. this should only be called by cookie stores. any other
    /// uses of this method will result in the cookie not getting
    /// correctly deserialized on subsequent requests.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// # fn main() -> async_session::Result { let rt = tokio::runtime::Runtime::new().unwrap();
    /// let handle = rt.handle();
    /// handle.block_on(async {
    /// let mut session = Session::new();
    /// session.set_cookie_value("hello".to_owned());
    /// let cookie_value = session.into_cookie_value().unwrap();
    /// assert_eq!(cookie_value, "hello".to_owned());
    /// # Ok(()) }) }
    /// ```
    pub fn set_cookie_value(&mut self, cookie_value: String) {
        self.inner.set_cookie_value(cookie_value)
    }

    /// returns the expiry timestamp of this session, if there is one
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// # fn main() -> async_session::Result { let rt = tokio::runtime::Runtime::new().unwrap();
    /// let handle = rt.handle();
    /// handle.block_on(async {
    /// let mut session = Session::new();
    /// assert_eq!(None, session.expiry());
    /// session.expire_in(std::time::Duration::from_secs(1));
    /// assert!(session.expiry().is_some());
    /// # Ok(()) }) }
    /// ```
    pub fn expiry(&self) -> Option<&DateTime<Utc>> {
        self.inner.expiry()
    }

    /// assigns an expiry timestamp to this session
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// # fn main() -> async_session::Result { let rt = tokio::runtime::Runtime::new().unwrap();
    /// let handle = rt.handle();
    /// handle.block_on(async {
    /// let mut session = Session::new();
    /// assert_eq!(None, session.expiry());
    /// session.set_expiry(chrono::Utc::now());
    /// assert!(session.expiry().is_some());
    /// # Ok(()) }) }
    /// ```
    pub fn set_expiry(&mut self, expiry: DateTime<Utc>) {
        self.inner.set_expiry(expiry)
    }

    /// assigns the expiry timestamp to a duration from the current time.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// # fn main() -> async_session::Result { let rt = tokio::runtime::Runtime::new().unwrap();
    /// let handle = rt.handle();
    /// handle.block_on(async {
    /// let mut session = Session::new();
    /// assert_eq!(None, session.expiry());
    /// session.expire_in(std::time::Duration::from_secs(1));
    /// assert!(session.expiry().is_some());
    /// # Ok(()) }) }
    /// ```
    pub fn expire_in(&mut self, ttl: std::time::Duration) {
        self.inner.expire_in(ttl)
    }

    /// predicate function to determine if this session is
    /// expired. returns false if there is no expiry set, or if it is
    /// in the past.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// # use std::time::Duration;
    /// # fn main() -> async_session::Result { let rt = tokio::runtime::Runtime::new().unwrap();
    /// let handle = rt.handle();
    /// handle.block_on(async {
    /// let mut session = Session::new();
    /// assert_eq!(None, session.expiry());
    /// assert!(!session.is_expired());
    /// session.expire_in(Duration::from_secs(1));
    /// assert!(!session.is_expired());
    /// tokio::time::sleep(Duration::from_secs(2)).await;
    /// assert!(session.is_expired());
    /// # Ok(()) }) }
    /// ```
    pub fn is_expired(&self) -> bool {
        self.inner.is_expired()
    }

    /// Ensures that this session is not expired. Returns None if it is expired
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// # use std::time::Duration;
    /// # fn main() -> async_session::Result { let rt = tokio::runtime::Runtime::new().unwrap();
    /// let handle = rt.handle();
    /// handle.block_on(async {
    /// let session = Session::new();
    /// let mut session = session.validate().unwrap();
    /// session.expire_in(Duration::from_secs(1));
    /// let session = session.validate().unwrap();
    /// tokio::time::sleep(Duration::from_secs(2)).await;
    /// assert_eq!(None, session.validate());
    /// # Ok(()) }) }
    /// ```
    pub fn validate(self) -> Option<Self> {
        if self.inner.is_expired() {
            None
        } else {
            Some(Self {
                inner: self.inner,
                regenerate: self.regenerate,
            })
        }
    }

    /// Checks if the data has been modified. This is based on the
    /// implementation of [`PartialEq`] for the inner data type.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// # fn main() -> async_session::Result { let rt = tokio::runtime::Runtime::new().unwrap();
    /// let handle = rt.handle();
    /// handle.block_on(async {
    /// let mut session = Session::new();
    /// assert!(!session.data_changed(), "new session is not changed");
    /// session.insert("key", 1);
    /// assert!(session.data_changed());
    ///
    /// session.reset_data_changed();
    /// assert!(!session.data_changed());
    /// session.remove("key");
    /// assert!(session.data_changed());
    /// # Ok(()) }) }
    /// ```
    pub fn data_changed(&self) -> bool {
        self.inner.data_changed()
    }

    /// Resets `data_changed` dirty tracking. This is unnecessary for
    /// any session store that serializes the data to a string on
    /// storage.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// # fn main() -> async_session::Result { let rt = tokio::runtime::Runtime::new().unwrap();
    /// let handle = rt.handle();
    /// handle.block_on(async {
    /// let mut session = Session::new();
    /// assert!(!session.data_changed(), "new session is not changed");
    /// session.insert("key", 1);
    /// assert!(session.data_changed());
    ///
    /// session.reset_data_changed();
    /// assert!(!session.data_changed());
    /// session.remove("key");
    /// assert!(session.data_changed());
    /// # Ok(()) }) }
    /// ```
    pub fn reset_data_changed(&self) {
        self.inner.reset_data_changed()
    }

    /// Ensures that this session is not expired. Returns None if it is expired
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// # use std::time::Duration;
    /// # fn main() -> async_session::Result { let rt = tokio::runtime::Runtime::new().unwrap();
    /// let handle = rt.handle();
    /// handle.block_on(async {
    /// let mut session = Session::new();
    /// session.expire_in(Duration::from_secs(123));
    /// let expires_in = session.expires_in().unwrap();
    /// assert!(123 - expires_in.as_secs() < 2);
    /// # Ok(()) }) }
    /// ```
    /// Duration from now to the expiry time of this session
    pub fn expires_in(&self) -> Option<std::time::Duration> {
        self.inner.expires_in()
    }

    /// takes the cookie value and consume this session.
    /// this is generally only performed by the session store
    ///
    /// # Example
    ///
    /// ```rust
    /// # use async_session::Session;
    /// # fn main() -> async_session::Result { let rt = tokio::runtime::Runtime::new().unwrap();
    /// let handle = rt.handle();
    /// handle.block_on(async {
    /// let mut session = Session::new();
    /// session.set_cookie_value("hello".to_owned());
    /// let cookie_value = session.into_cookie_value().unwrap();
    /// assert_eq!(cookie_value, "hello".to_owned());
    /// # Ok(()) }) }
    /// ```
    pub fn into_cookie_value(self) -> Option<String> {
        self.inner.into_cookie_value()
    }

    pub(crate) fn into_inner(self) -> async_session::Session {
        self.inner
    }

    pub(crate) fn should_regenerate(&self) -> bool {
        self.regenerate.load(Ordering::Relaxed)
    }

    pub(crate) fn from_inner(inner: async_session::Session) -> Self {
        Self {
            inner,
            regenerate: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) fn inner_regenerate(&mut self) {
        self.inner.regenerate();
    }
}
