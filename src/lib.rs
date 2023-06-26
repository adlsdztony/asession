use cookie_store::CookieStore;
use log::{error, warn};
use reqwest::Client;
use reqwest_cookie_store::CookieStoreMutex;
use std::fs::{self, File};
use std::io::BufReader;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

/// a builder to build a session
pub struct SessionBuilder {
    cookie_store_path: Option::<PathBuf>,
}

impl SessionBuilder {
    pub fn new() -> SessionBuilder {
        SessionBuilder {
            cookie_store_path: Option::<PathBuf>::None,
        }
    }

    /// set path to store cookies
    pub fn cookies_store_into(mut self, cookie_store_path: PathBuf) -> SessionBuilder {
        self.cookie_store_path = Some(cookie_store_path);
        self
    }

    pub fn build(self) -> anyhow::Result<Session> {
        Session::try_new(self.cookie_store_path)
    }
}


/// `Session` is a user-friendly `Client` wrapper, which automatically handles cookies and load/store
/// cookies from/to the specified path.
#[derive(Debug, Clone)]
pub struct Session {
    #[allow(dead_code)] // just make clippy happy
    state: Arc<State>,
    client: Client,
}

impl Session {

    /// Try to creates a new `Session` instance, and load cookies from `cookie_store_path`.
    pub fn try_new(cookie_store_path: Option<PathBuf>) -> anyhow::Result<Session> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36 Edg/115.0.0.0".parse().unwrap());
        
        let state = State::try_new(cookie_store_path)?;
        let state = Arc::new(state);

        let client = Client::builder()
            .cookie_provider(state.cookie_store.clone())
            .default_headers(headers)
            .build()?;

        Ok(Session { state, client })
    }

    /// Get the cookie store of this session.
    pub fn get_cookie_store(&self) -> Arc<CookieStoreMutex> {
        self.state.cookie_store.clone()
    }
}

impl Deref for Session {
    type Target = Client;
    fn deref(&self) -> &Client {
        &self.client
    }
}

#[derive(Debug)]
pub struct State {
    cookie_store_path: Option<PathBuf>,
    cookie_store: Arc<CookieStoreMutex>,
}

impl State {
    pub fn try_new(cookie_store_path: Option<PathBuf>) -> anyhow::Result<State> {
        if cookie_store_path.is_none() {
            return Ok(State {
                cookie_store_path,
                cookie_store: Arc::new(CookieStoreMutex::new(CookieStore::default())),
            });
        }

        let cookie_store_path = cookie_store_path.unwrap();

        let cookie_store = match File::open(&cookie_store_path) {
            Ok(f) => CookieStore::load_json(BufReader::new(f)).map_err(|e| {
                let context = format!(
                    "error when read cookies from {}",
                    cookie_store_path.display()
                );
                anyhow::anyhow!("{}", e).context(context)
            })?,
            Err(e) => {
                warn!(
                    "open {} failed. error: {}, use default empty cookie store",
                    cookie_store_path.display(),
                    e
                );
                CookieStore::default()
            }
        };
        let cookie_store = Arc::new(CookieStoreMutex::new(cookie_store));

        Ok(State {
            cookie_store_path: Some(cookie_store_path),
            cookie_store,
        })
    }

    
        
}

impl Drop for State {
    /// When `State` is dropped, store cookies to `cookie_store_path`.
    fn drop(&mut self) {
        if self.cookie_store_path.is_none() {
            return;
        }
        let cookie_store_path = self.cookie_store_path.as_ref().unwrap();
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&cookie_store_path)
        {
            Ok(f) => f,
            Err(e) => {
                error!(
                    "open {} for write failed. error: {}",
                    cookie_store_path.display(),
                    e
                );
                return;
            }
        };

        let store = self.cookie_store.lock().unwrap();
        if let Err(e) = store.save_json(&mut file) {
            error!(
                "save cookies to path {} failed. error: {}",
                cookie_store_path.display(),
                e
            );
        }
    }
}