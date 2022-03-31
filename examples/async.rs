use futures::{channel::mpsc::UnboundedReceiver, Stream, StreamExt};
use netlink_packet_core::{
    NetlinkDeserializable, NetlinkMessage, NetlinkPayload, NLM_F_ACK, NLM_F_REQUEST,
};
use netlink_proto::{new_connection, ConnectionHandle};
use netlink_sys::{protocols::NETLINK_CONNECTOR, SocketAddr};
use w1_netlink::proto::{
    command::W1NetlinkCommand, connector::NlConnectorMessage, message::W1NetlinkMessage,
};

struct W1Provider {
    handle: ConnectionHandle<NlConnectorMessage<W1NetlinkMessage>>,
    messages: UnboundedReceiver<(
        NetlinkMessage<NlConnectorMessage<W1NetlinkMessage>>,
        SocketAddr,
    )>,
}

impl W1Provider {
    pub fn connect() -> Self {
        let (conn, handle, messages) =
            new_connection(NETLINK_CONNECTOR).expect("failed to create connection");
        tokio::task::spawn(async move {
            conn.await;
            println!("CONNECTION TASK EXITED!");
        });

        Self { handle, messages }
    }

    pub async fn list_masters(&mut self) -> Vec<u32> {
        let msg = W1NetlinkMessage::ListMasters(None);

        let _ = self.request(msg);

        println!("Sent. Receiving response.");

        let deserialized_message = self.receive().await;
        if let W1NetlinkMessage::ListMasters(Some(master_ids)) = deserialized_message {
            return master_ids;
        }
        unimplemented!()
    }

    pub async fn search(&mut self, master_id: u32) {
        let msg = W1NetlinkMessage::MasterCommand {
            target: master_id,
            cmds: vec![W1NetlinkCommand::Search(None)],
        };

        let _ = self.request(msg);
        let message = self.receive().await;
        println!("{:?}", message)
    }

    fn request(
        &mut self,
        message: W1NetlinkMessage,
    ) -> impl Stream<Item = NetlinkMessage<NlConnectorMessage<W1NetlinkMessage>>> {
        let kernel_unicast = SocketAddr::new(0, 0);
        let cmsg = NlConnectorMessage::new(0, [message]);

        let mut nl_msg = NetlinkMessage::from(cmsg);
        nl_msg.header.port_number = std::process::id();
        nl_msg.header.flags = NLM_F_ACK | NLM_F_REQUEST;

        self.handle.request(nl_msg, kernel_unicast).unwrap()
    }

    async fn receive(&mut self) -> W1NetlinkMessage {
        if let Some((message, _addr)) = self.messages.next().await {
            println!("got event: {:?}", message);

            if let NetlinkPayload::Done(Some(bytes)) = message.payload {
                println!("{:02x?}", bytes);
                let deserialized_message =
                    NlConnectorMessage::<W1NetlinkMessage>::deserialize(&message.header, &bytes)
                        .unwrap();
                println!("{:?}", deserialized_message);
                return deserialized_message.into_iter().next().unwrap();
            }
        }
        unimplemented!()
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut provider = W1Provider::connect();
    let masters_list = provider.list_masters().await;
    println!("{:?}", masters_list);
    provider.search(masters_list[0]).await;
}
