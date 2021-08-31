use std::collections::HashMap;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::sync::{
    oneshot,
    mpsc,
    Mutex as TokioMutex
};
use tokio::time::sleep;

use crate::cache::AniDbCache;
use crate::requests::{
    AniDbRequest,
    auth::AuthRequest
};
use crate::errors::AniDbError;
use crate::{
    CLIENT_NAME,
    CLIENT_VERSION,
    ANIDB_ADDR,
};

// helper for processing data
macro_rules! expect_next {
    ($iter:ident, $offset:ident) => {
        match $iter.next() {
            Some(x) => x,
            None => {
                eprintln!("Invalid data from tag");
                $offset = 0;
                continue;
            }
        };
    }
}

#[derive(Clone)]
pub struct AniDbClient<C: AniDbCache> {
    socket: Arc<UdpSocket>,
    request_map: Arc<Mutex<HashMap<String, oneshot::Sender<(String, String, String)>>>>,
    request_queue: mpsc::Sender<String>,
    cache: Arc<TokioMutex<C>>,
    client_name: String,
    client_version: i32,
    username: String,
    password: String,
    next_request_id: Arc<Mutex<u64>>,
    session_id: Arc<TokioMutex<Option<String>>>
}

impl<C> AniDbClient<C> where C: AniDbCache {
    pub async fn new(
        cache: C,
        user: &str,
        pass: &str
    ) -> Result<AniDbClient<C>, AniDbError> {
        Self::new_with_client(cache, user, pass, &CLIENT_NAME, CLIENT_VERSION)
            .await
    }

    pub async fn new_with_client(
        cache: C,
        user: &str,
        pass: &str,
        client_id: &str,
        client_version: i32,
    ) -> Result<AniDbClient<C>, AniDbError> {
        let socket = UdpSocket::bind(("0.0.0.0", 9000)).await?;
        let socket = Arc::new(socket);
        let cache = Arc::new(TokioMutex::new(cache));
        let (tx, mut rx) = mpsc::channel::<String>(100);
        {
            let socket = socket.clone();
            tokio::spawn(async move {
                // start at 1 to assume login was first
                let mut packet_count: u64 = 1;
                while let Some(s) = rx.recv().await {
                    dbg!(&s);
                    let data = s.as_bytes();
                    match socket.send(data).await {
                        Ok(sent) => assert_eq!(data.len(), sent),
                        Err(e) => {
                            dbg!(e);
                            break;
                        }
                    }
                    // rate limit sending (TODO: make optional?)
                    println!("anidb sent {} bytes", data.len());
                    packet_count += 1;
                    if packet_count > 5 {
                        sleep(Duration::from_secs(2)).await;
                    }
                }
            });
        }
        let request_map: Arc<Mutex<HashMap<String, oneshot::Sender<(String, String, String)>>>>
            = Arc::new(Mutex::new(HashMap::new()));
        {
            let socket = socket.clone();
            let request_map = request_map.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 2048];
                let mut offset = 0;
                loop {
                    match socket.recv(&mut buf[offset..]).await {
                        Ok(len) => {
                            let buf_len = offset + len;
                            if !buf[..buf_len].ends_with(b"\n") {
                                // we expect more bytes?
                                eprintln!(
                                    "expecting more bytes after: {}",
                                    match std::str::from_utf8(&buf[..buf_len]) {
                                        Ok(s) => s,
                                        Err(_e) => "failed to decode as utf8"
                                    }
                                );
                                offset = buf_len;
                                continue;
                            }
                            println!("anidb received {} bytes");
                            match std::str::from_utf8(&buf[..buf_len]) {
                                Ok(data) => {
                                    let mut data_iter = data.splitn(2, " ");
                                    let tag = expect_next!(data_iter, offset);
                                    // TODO: now we have tag, we can potentially reply
                                    let data = expect_next!(data_iter, offset);
                                    let mut data_iter = data.splitn(2, " ");
                                    let code = expect_next!(data_iter, offset).to_string();
                                    let data = expect_next!(data_iter, offset);
                                    let mut data_iter = data.splitn(2, "\n");
                                    let reply = expect_next!(data_iter, offset).to_string();
                                    let data = expect_next!(data_iter, offset).to_string();
                                    let request_sender = {
                                        request_map.lock().unwrap()
                                            .remove(tag)
                                    };
                                    if let Some(request_sender) = request_sender {
                                        match request_sender.send((code, reply, data)) {
                                            Ok(_) => (),
                                            Err(e) => {
                                                eprintln!("failed to respond to request `{}`: {:?}", tag, e);
                                            }
                                        }
                                    }
                                    offset = 0;
                                },
                                Err(_e) => {
                                    eprintln!("failed to decode buffer as utf8");
                                }
                            }
                        },
                        Err(e) => {
                            dbg!(e);
                            break;
                        }
                    }
                }
            });
        }

        Ok(AniDbClient {
            socket,
            cache,
            username: user.to_owned(),
            password: pass.to_owned(),
            request_map,
            request_queue: tx,
            next_request_id: Arc::new(Mutex::new(0)),
            client_name: client_id.to_owned(),
            client_version,
            session_id: Arc::new(TokioMutex::new(None)),
        })
    }

    async fn connect(
        &self,
    ) -> Result<String, AniDbError> {
        self.socket.connect(ANIDB_ADDR).await?;
        let auth = AuthRequest::builder()
            .user(self.username.clone())
            .pass(self.password.clone())
            .client(self.client_name.clone())
            .clientver(self.client_version)
            .build();
        let tag = self.next_tag();
        let req_str = format!(
            "{} {}&tag={}",
            AuthRequest::name(),
            auth.encode()?,
            tag
        );
        let (sender, receiver) = oneshot::channel();
        {
            let mut map = self.request_map.lock().unwrap();
            map.insert(tag, sender);
        }
        self.request_queue.send(req_str).await?;
        let (code, resp_str, data) = receiver.await?;
        let resp = auth.decode_response(&code, &resp_str, &data)?;
        Ok(resp.session_id)
    }

    // fn get_session_id(&self) -> Option<String> {
    //     self.session_id.lock().unwrap().clone()
    // }

    fn next_tag(&self) -> String {
        let mut next_request_id = self.next_request_id.lock().unwrap();
        let tag = format!("t{:x}", *next_request_id);
        *next_request_id += 1;
        tag
    }

    async fn get_session_id_or_connect(&self) -> Result<String, AniDbError> {
        let mut sid = self.session_id.lock().await;
        match sid.as_ref() {
            Some(session_id) => Ok(session_id.clone()),
            None => {
                let session_id = self.connect().await?;
                *sid = Some(session_id.clone());
                Ok(session_id)
            }
        }
    }

    pub async fn request<R>(
        &self,
        request: R
    ) -> Result<R::Response, R::Error>
    where R: AniDbRequest
    {
        let args: String = request.encode()?;
        if let Some((code, resp_str, data)) = {
            println!("Locking anidb cache");
            let cache = self.cache.lock().await;
            cache.get(R::name(), &args).await
                .map_err(|e| AniDbError::CacheError(format!("{}", e)))?
        } {
            println!("lock released");
            request.decode_response(&code, &resp_str, &data)
        } else {
            println!("lock released");
            let (tag, req_str) = if R::requires_login() {
                let session_id = self.get_session_id_or_connect().await?;
                let tag = self.next_tag();
                (tag.clone(), format!(
                    "{} {}&tag={}&s={}",
                    R::name(), args, tag, session_id
                ))
            } else {
                let tag = self.next_tag();
                (tag.clone(), format!(
                    "{} {}&tag={}",
                    R::name(), args, tag
                ))
            };
            let (sender, receiver) = oneshot::channel();
            {
                println!("Locking anidb request map");
                let mut map = self.request_map.lock().unwrap();
                map.insert(tag, sender);
            }
            println!("unlocked anidb request map ... sending request");
            self.request_queue.send(req_str).await?;
            println!("waiting for reply");
            let (code, reply, data) = receiver.await?;
            {
                println!("Locking anidb cache 2");
                let cache = self.cache.lock().await;
                cache.store(
                    R::name(), &args, &code, &reply, &data
                ).await
                    .map_err(|e| AniDbError::CacheError(format!("{}", e)))?
            }
            println!("decoding response");
            let resp = request.decode_response(&code, &reply, &data)?;
            Ok(resp)
        }
    }
}
