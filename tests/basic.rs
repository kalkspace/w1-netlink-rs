use netlink_packet_core::NetlinkMessage;
use w1_netlink::{
    command::{W1Command, W1NetlinkCommand},
    message::{W1MessageType, W1NetlinkMessage},
    NlConnectorMessage,
};

#[test]
fn serialize() {
    let cmd = W1NetlinkCommand::new(W1Command::Search, ());
    let msg = W1NetlinkMessage::new(W1MessageType::MasterCmd, 0, cmd);
    let cmsg = NlConnectorMessage::new(0, msg);

    let mut packet = NetlinkMessage::from(cmsg);
    packet.finalize();

    let mut buf = vec![0; packet.header.length as usize];
    packet.serialize(&mut buf[..]);
}
