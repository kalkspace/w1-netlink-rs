use futures::StreamExt;
use netlink_packet_core::{NetlinkMessage, NLMSG_DONE};
use netlink_proto::new_connection;
use netlink_sys::{protocols::NETLINK_CONNECTOR, SocketAddr};
use w1_netlink::proto::{
    command::W1NetlinkCommand,
    connector::NlConnectorMessage,
    message::{TargetId, W1MessageType, W1NetlinkMessage},
};

#[tokio::test]
async fn write_req() {
    let kernel_unicast = SocketAddr::new(0, 0);

    let (conn, mut handle, mut messages) = new_connection(NETLINK_CONNECTOR).expect("");
    tokio::spawn(conn);

    let cmd = W1NetlinkCommand::Search(None);
    let msg = W1NetlinkMessage::new(W1MessageType::MasterCmd, TargetId::master_id(1), [cmd]);
    let cmsg = NlConnectorMessage::new(0, [msg]);

    let mut nl_msg = NetlinkMessage::from(cmsg);
    nl_msg.header.message_type = NLMSG_DONE;
    nl_msg.header.port_number = std::process::id();

    let mut stream = handle.request(nl_msg, kernel_unicast).unwrap();
    println!("Sent. Receiving response.");
    while let Some(msg) = stream.next().await {
        println!("got msg: {:?}", msg);
    }

    println!("Receiving messages...");
    while let Some((message, _addr)) = messages.next().await {
        println!("got event: {:?}", message);
    }
}
