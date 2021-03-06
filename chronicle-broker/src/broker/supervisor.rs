use super::zmq;
use chronicle_common::{
    actor,
    traits::launcher::LauncherTx,
};
use std::string::ToString;
use tokio::sync::mpsc;
actor!(SupervisorBuilder {
    sn: Option<Vec<String>>,
    trytes: Option<Vec<String>>,
    sn_trytes: Option<Vec<String>>,
    launcher_tx: Box<dyn LauncherTx>
});
pub enum Event {
    // TODO useful events to dyanmicly add/remove zmq nodes
}
pub type Sender = mpsc::UnboundedSender<Event>;
pub type Receiver = mpsc::UnboundedReceiver<Event>;

pub struct Peer {
    topic: Topic,
    address: String,
    connected: bool,
}
impl Peer {
    pub fn get_address<'a>(&'a self) -> &'a str {
        &self.address
    }
    pub fn get_topic(&self) -> Topic {
        self.topic
    }
    pub fn get_topic_as_string(&self) -> String {
        self.topic.to_string()
    }
}

#[derive(Clone, Copy)]
pub enum Topic {
    Sn,
    Trytes,
    SnTrytes,
}
impl ToString for Topic {
    fn to_string(&self) -> String {
        match self {
            Topic::Sn => "sn".to_owned(),
            Topic::Trytes => "trytes".to_owned(),
            Topic::SnTrytes => "sn_trytes".to_owned(),
        }
    }
}

impl SupervisorBuilder {
    pub fn build(mut self) -> Supervisor {
        let mut peers = Vec::new();
        // create peers from sn nodes (if any)
        if let Some(mut addresses) = self.sn.unwrap().take() {
            for address in addresses.drain(..) {
                peers.push(Peer {
                    topic: Topic::Sn,
                    address,
                    connected: false,
                })
            }
        }
        // create peers from trytes nodes (if any)
        if let Some(mut addresses) = self.trytes.unwrap().take() {
            for address in addresses.drain(..) {
                peers.push(Peer {
                    topic: Topic::Trytes,
                    address,
                    connected: false,
                })
            }
        }
        // create peers from sn_trytes nodes (if any)
        if let Some(mut addresses) = self.sn_trytes.unwrap().take() {
            for address in addresses.drain(..) {
                peers.push(Peer {
                    topic: Topic::SnTrytes,
                    address,
                    connected: false,
                })
            }
        }
        let (tx, rx) = mpsc::unbounded_channel::<Event>();
        Supervisor {
            peers,
            tx,
            rx,
            launcher_tx: self.launcher_tx.unwrap(),
        }
    }
}
pub struct Supervisor {
    peers: Vec<Peer>,
    tx: Sender,
    rx: Receiver,
    launcher_tx: Box<dyn LauncherTx>,
}

impl Supervisor {
    pub async fn run(mut self) {
        for peer in self.peers {
            let zmq_worker = zmq::ZmqBuilder::new().peer(peer).supervisor_tx(self.tx.clone()).build();
        }
        // TODO await exit signal from zmq workers or dynamic topology events from dashboard
    }
}
