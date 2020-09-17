# kcp client
thread sale kcp client

# Examples Echo
```rust
use kcpclient::*;
use std::error::Error;
use std::sync::Arc;
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;


fn main() {
    let mut config = KcpConfig::default();
    config.nodelay = Some(KcpNoDelayConfig::fastest());

    let kcp_client = KcpClient::connect("127.0.0.1:5555", config).unwrap();
    kcp_client.init_conv().unwrap();
    let is_run =Arc::new( AtomicBool::new(true));
    let kcp_client = Arc::new(kcp_client);
    let up = kcp_client.clone();
    let run_arc=is_run.clone();
    let th=thread::spawn(move || {
        while run_arc.load(Ordering::Acquire) {
            up.update().unwrap();
            sleep(Duration::from_millis(10));
         }
    });

     for _ in 0..100 {
         kcp_client.send(b"123123123123").unwrap();
         let res = kcp_client.recv();
         match res {
             Ok(data)=>{
                println!("{}", String::from_utf8_lossy(&data));
             },
             Err(er)=>{
                println!("{}",er);
            }
         }
     }

     is_run.store(false,Ordering::Release);
    th.join().unwrap();
 }
```