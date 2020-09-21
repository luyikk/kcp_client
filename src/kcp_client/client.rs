use std::net::{UdpSocket, ToSocketAddrs};
use std::error::Error;
use std::sync::{Arc, Mutex};
use crate::kcp::Kcp;
use bytes::{Bytes, Buf};
use crate::kcp::kcp_config::KcpConfig;
use crate::kcp::Error as KCPError;

/// KCP 客户端
pub struct KcpClient{
    udp_client:Arc<UdpSocket>,
    kcp:Mutex<Kcp>
}

impl KcpClient{
    /// 连接
    pub fn connect<A:ToSocketAddrs>(addr:A,config:KcpConfig)->Result<KcpClient,Box<dyn Error>>{
        let udp_client=Arc::new(UdpSocket::bind("0.0.0.0:0")?);
        udp_client.connect(addr)?;
        let mut kcp=Kcp::new_stream(0,udp_client.clone());
        config.apply_config(&mut kcp);
        let  kcp=Mutex::new(kcp);
        Ok(KcpClient{
            udp_client,
            kcp
        })
    }

    /// 从远程服务器获取并设置conv 发送4字节令牌,收到8字节 4字节令牌+4字节CONV
    pub fn init_conv(&self) ->Result<u32,Box<dyn Error+'_>> {
        if self.check_conv(){
            return Err("conv already set".into());
        }

        if self.udp_client.send(&[1, 2, 3, 4])? != 4 {
            return Err("not send data".into());
        }
        let mut  conv_data = [0; 8];
        let len=self.udp_client.recv(&mut conv_data)?;
        if len !=8{
            return Err("recv data err".into());
        }
        let mut read=Bytes::from(conv_data.to_vec());
        read.get_u32_le();
        let conv=read.get_u32_le();
        self.set_conv(conv)?;
        Ok(conv)
    }

    /// 手动设置conv
    pub fn set_conv(&self, conv:u32) ->Result<(),Box<dyn Error+'_>> {
        let mut kcp = self.kcp.lock()?;

        if kcp.conv()!=0{
            return Err("conv already set".into());
        }
        kcp.set_conv(conv);
        Ok(())
    }

    /// 检查conv是否设置如果0 就是没有设置返回false
    pub fn check_conv(&self) ->bool{
        let  res = self.kcp.lock();
        if let Ok(kcp)=res {
            if kcp.conv() != 0 {
                true
            } else {
                false
            }
        }else { false }
    }

    /// 读取数据包
    pub fn recv(&self)->Result<Bytes,Box<dyn Error+'_>>{
       loop {
          let res:Option<Bytes>= {
              let mut kcp = self.kcp.lock()?;
              let res = kcp.peeksize();
              if let Ok(len)=res {
                  let mut buffer = vec![0; len];
                  kcp.recv(&mut buffer)?;
                  Some(Bytes::from(buffer))
              }else{
                  None
              }
          };

         return  match res {
               Some(res)=>Ok(res),
               None=>{
                   let mut buffer = [0; 4096];
                   let len = self.udp_client.recv(&mut buffer)?;
                   let mut kcp = self.kcp.lock()?;
                   kcp.input(&buffer[..len])?;
                   let len = kcp.peeksize()?;
                   return if len > 0 {
                       let mut buffer = vec![0; len];
                       kcp.recv(&mut buffer)?;
                       Ok(Bytes::from(buffer))
                   } else {
                       Err(KCPError::RecvQueueEmpty.into())
                   }
               }
           };
       }
    }

    /// 获取当前时间戳 转换为u32
    #[inline]
    fn current() -> u32 {
        let time =chrono::Local::now().timestamp_millis() & 0xffffffff;
        time as u32
    }

    /// 更新
    pub fn update(&self)->Result<(),Box<dyn Error+'_>>{
        let mut kcp= self.kcp.lock()?;
        kcp.update(Self::current())?;
        Ok(())
    }

    /// 发送
    pub fn send(&self,data:&[u8])->Result<usize,Box<dyn Error+'_>>{
        let mut kcp= self.kcp.lock()?;
        Ok(kcp.send(data)?)
    }


}