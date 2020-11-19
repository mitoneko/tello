// Telloの制御
use crate::error::TelloError;
use array_macro::*;
use std::net::{ToSocketAddrs, UdpSocket};
use std::num::Wrapping;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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
    timeout_sec: u16,
}

impl Controller {
    /// コントローラーを立ち上げる。
    /// # 引数
    ///  - tello_ip : Telloのipアドレス、及び、ポート番号。
    pub fn new_with_ip(tello_ip: impl ToSocketAddrs) -> Result<Self, TelloError> {
        let socket = UdpSocket::bind(TELLO_CMD_BIND)?;
        socket.connect(tello_ip)?;
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (ret_tx, ret_rx) = mpsc::channel();
        thread::spawn(move || {
            Self::send_proc(socket, cmd_rx, ret_tx);
        });

        Ok(Self {
            cmd_sender: cmd_tx,
            ret_receiver: ret_rx,
            next_job_no: Wrapping(1u16),
            job_rets: array![JobRet { id: 0, ret: Ok(0) }; JOB_RETS_SIZE],
            job_rets_cur_idx: 0,
            timeout_sec: 30,
        })
    }

    /// コントローラーを立ち上げる
    ///
    /// Telloのipは、"192.168.10.1:8889"とする。
    pub fn new() -> Result<Self, TelloError> {
        Controller::new_with_ip(TELLO_CMD_IP)
    }

    /// 同期実行時のTelloからのレスポンスのタイムアウト秒数の設定
    /// デフォルトは、30秒
    pub fn set_timeout_sec(&mut self, sec: u16) {
        self.timeout_sec = sec;
    }

    pub fn exec_cmd(&mut self, cmd: TelloCommand) -> Result<u32, TelloError> {
        {
            let cmd = cmd.clone();
            let job = Job {
                id: self.next_job_no.0,
                cmd,
            };
            self.cmd_sender.send(job).expect("コマンド送信パイプエラー");
        }
        let mut ret_val = Err(TelloError::TelloTimeout(format!("CMD[{}]", cmd)));
        for _i in 0..self.timeout_sec * 10 {
            self.recv_job_ret();
            if let Some(ret) = self.find_job_rets(self.next_job_no.0) {
                ret_val = ret.ret.clone();
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
        self.next_job_no += Wrapping(1);
        ret_val
    }

    fn recv_job_ret(&mut self) {
        while let Ok(ret) = self.ret_receiver.try_recv() {
            self.add_job_rets(ret);
        }
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
            idx = match idx {
                0 => JOB_RETS_SIZE - 1,
                n => n - 1,
            }
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
                    ret: Err(e.into()),
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
                Err(e) => Err(e.into()),
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

#[derive(Debug, Clone)]
pub enum TelloCommand {
    Command,
    Takeoff,
    Land,
}

impl std::fmt::Display for TelloCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TelloCommand::*;
        match self {
            Command => write!(f, "command"),
            Takeoff => write!(f, "takeoff"),
            Land => write!(f, "land"),
        }
    }
}
