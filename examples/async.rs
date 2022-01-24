use futures::StreamExt;
use netlink_packet_core::{NetlinkMessage, NLM_F_ACK, NLM_F_REQUEST};
use netlink_proto::new_connection;
use netlink_sys::{protocols::NETLINK_CONNECTOR, SocketAddr};
use w1_netlink::proto::{
    connector::NlConnectorMessage,
    message::{MasterId, TargetId, W1MessageType, W1NetlinkMessage},
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let kernel_unicast = SocketAddr::new(0, 0);

    let (conn, mut handle, mut messages) =
        new_connection(NETLINK_CONNECTOR).expect("failed to create connection");
    tokio::task::spawn(async move {
        conn.await;
        println!("CONNECTION TASK EXITED!");
    });

    tokio::spawn(async move {
        let msg = W1NetlinkMessage::<MasterId>::new(
            W1MessageType::ListMasters,
            TargetId::master_id(0),
            [],
        );
        let cmsg = NlConnectorMessage::new(0, [msg]);

        let mut nl_msg = NetlinkMessage::from(cmsg);
        nl_msg.header.port_number = std::process::id();
        nl_msg.header.flags = NLM_F_ACK | NLM_F_REQUEST;

        let mut stream = handle.request(nl_msg, kernel_unicast).unwrap();
        println!("Sent. Receiving response.");
        while let Some(msg) = stream.next().await {
            println!("got msg: {:?}", msg);
        }
    });

    println!("Receiving messages...");
    while let Some((message, _addr)) = messages.next().await {
        println!("got event: {:?}", message);
    }
}
