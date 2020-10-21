/// ステータス取得の管理
use super::data::StatusData;
use crate::error::TelloError;
use std::net::UdpSocket;
use std::str;
use std::sync::mpsc;
use std::thread;

/// Telloのステータス受信とデータの管理
#[derive(Debug)]
pub struct Manager {
    data: StatusData,
    rx: mpsc::Receiver<StatusData>,
}

impl Manager {
    /// Manageの生成。
    ///
    /// # 引数
    /// Telloステータス受信用ソケットか、None。
    /// Noneの場合、デフォルトとして"0.0.0.0:8890"のポートを使用する。
    ///
    pub fn new(socket: impl Into<Option<UdpSocket>>) -> Result<Self, TelloError> {
        let socket = socket.into().unwrap_or(UdpSocket::bind("0.0.0.0:8890")?);
        // データ受信スレッドの生成
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            Self::recieve_proc(socket, tx);
        });

        Ok(Self {
            data: StatusData::default(),
            rx,
        })
    }

    /// 【内部関数】ステータス受信スレッドの本体。
    fn recieve_proc(socket: UdpSocket, tx: mpsc::Sender<StatusData>) -> ! {
        loop {
            let mut stat_buf = [0; 1024];
            let (len, _addr) = socket.recv_from(&mut stat_buf).unwrap_or_else(|e| {
                eprintln!("ステータス受信ユニット:ソケット受信エラー->{:?}", e);
                std::process::exit(1);
            });
            let stat: StatusData = str::from_utf8(&stat_buf[0..len]).unwrap().parse().unwrap();
            tx.send(stat).unwrap_or_else(|e| {
                eprintln!("ステータス受信ユニット:プロセス通信エラー{:?}", e);
                std::process::exit(1);
            });
        }
    }

    /// 受信した最新のステータスデータを返す。
    /// 内部ステータスデータ構造体を最新データに更新するため、mutが必要。
    pub fn get_data(&mut self) -> StatusData {
        // メッセージの受信とデータ更新
        for rx_data in self.rx.try_iter() {
            self.data = rx_data;
        }

        self.data.clone()
    }
}
