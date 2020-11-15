// Telloの制御
use crate::error::TelloError;
use array_macro::*;
use std::net::{ToSocketAddrs, UdpSocket};
use std::num::Wrapping;
use std::sync::mpsc;
use std::thread;

const TELLO_CMD_IP: &str = "192.168.10.1:8889";
const TELLO_CMD_BIND: &str = "0.0.0.0:0";
const JOB_RETS_SIZE: usize = 16;

/// Telloのコントローラー
#[derive(Debug)]
pub struct Controller {
    cmd_sender: mpsc::Sender<Job>,
    ret_receiver: mpsc::Receiver<JobRet>,
    next_job_no: Wrapping<u16>,
    job_rets: [JobRet; JOB_RETS_SIZE],
    job_rets_cur_idx: usize,
}

impl Controller {
    /// コントローラーを立ち上げる。
    /// # 引数
    ///  - socket : Telloとのコマンド送受信用UDPソケット。
    ///  - tello_ip : Telloのipアドレス、及び、ポート番号。
    ///
    ///  ともに、Noneを指定すればデフォルト値を使用する。
    pub fn new<U, A, A2>(socket: U, tello_ip: A) -> Result<Self, TelloError>
    where
        U: Into<Option<UdpSocket>>,
        A: Into<Option<A2>>,
        A2: ToSocketAddrs,
    {
        let socket = socket.into().unwrap_or(UdpSocket::bind(TELLO_CMD_BIND)?);
        match tello_ip.into() {
            Some(a) => socket.connect(a)?,
            None => socket.connect(TELLO_CMD_IP)?,
        }
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (ret_tx, ret_rx) = mpsc::channel();
        thread::spawn(move || {
            Self::send_proc(socket, cmd_rx, ret_tx);
        });

        Ok(Self {
            cmd_sender: cmd_tx,
            ret_receiver: ret_rx,
            next_job_no: Wrapping(0u16),
            job_rets: array![JobRet { id: 0, ret: Ok(0) }; JOB_RETS_SIZE],
            job_rets_cur_idx: 0,
        })
    }

    fn exec_cmd(&mut self, cmd: TelloCommand) -> Result<usize, TelloError> {
        unimplemented!()
    }

    fn recv_job_ret(&self) -> () {
        unimplemented!()
    }

    /// 戻り値バッファにリターン値を追加する。
    fn add_job_rets(&mut self, ret: JobRet) {
        let idx = (self.job_rets_cur_idx + 1) % JOB_RETS_SIZE;
        self.job_rets[idx] = ret;
        self.job_rets_cur_idx = idx;
    }

    /// jobのidよりリターン値を検索する。
    fn find_job_rets(&self, id: u16) -> Option<&JobRet> {
        let mut idx = self.job_rets_cur_idx;
        while idx != (self.job_rets_cur_idx + 1) % JOB_RETS_SIZE {
            if self.job_rets[idx].id == id {
                return Some(&self.job_rets[idx]);
            }
            idx = (idx - 1) % JOB_RETS_SIZE;
        }
        None
    }

    /// 内部関数: 指定されたコマンドをTelloに非同期で送信するスレッド本体
    ///
    /// # Panics
    /// 　送受信に使用するパイプにエラーが出るとパニックする。
    fn send_proc(
        tello_socket: UdpSocket,
        cmd_recv: mpsc::Receiver<Job>,
        ret_send: mpsc::Sender<JobRet>,
    ) -> ! {
        let mut buff = [0; 10];
        let err_recv = "コマンド送信部エラー。コマンド送信用パイプ不良";
        let err_send = "コマンド送信部エラー。コマンド結果返信用パイプ不良";
        loop {
            let Job { id, cmd } = cmd_recv.recv().expect(err_recv);
            if let Err(e) = tello_socket.send(cmd.to_string().as_bytes()) {
                let ret = JobRet {
                    id,
                    ret: Err(TelloError::SocketError(e)),
                };
                ret_send.send(ret).expect(err_send);
                continue;
            }
            let ret = match tello_socket.recv(&mut buff) {
                Ok(i) => match &buff[0..i] {
                    b"ok" => Ok(0),
                    b"error" => Err(TelloError::TelloCmdFail(cmd.to_string())),
                    _ => std::str::from_utf8(&buff[0..i])
                        .unwrap()
                        .parse::<u32>()
                        .or(Err(TelloError::TelloResponsIllegal(
                            String::from_utf8(buff[0..i].to_vec()).unwrap(),
                        ))),
                },
                Err(e) => Err(TelloError::SocketError(e)),
            };
            ret_send.send(JobRet { id, ret }).expect(err_send);
        }
    }
}

#[derive(Debug)]
struct Job {
    id: u16,
    cmd: TelloCommand,
}

#[derive(Debug)]
struct JobRet {
    id: u16,
    ret: Result<u32, TelloError>,
}

#[derive(Debug)]
pub enum TelloCommand {
    Command,
    Takeoff,
    Land,
}

impl ToString for TelloCommand {
    fn to_string(&self) -> String {
        use TelloCommand::*;
        match self {
            Command => "command".to_string(),
            Takeoff => "takeoff".to_string(),
            Land => "land".to_string(),
        }
    }
}
