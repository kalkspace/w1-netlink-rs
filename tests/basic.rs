use netlink_packet_core::NetlinkMessage;
use w1_netlink::{
    command::W1NetlinkCommand,
    connector::NlConnectorMessage,
    message::{TargetId, W1MessageType, W1NetlinkMessage},
};

#[test]
fn serialize() {
    let cmd = W1NetlinkCommand::Search;
    let msg = W1NetlinkMessage::new(W1MessageType::MasterCmd, TargetId::master_id(0), [cmd]);
    let cmsg = NlConnectorMessage::new(0, [msg]);

    let mut packet = NetlinkMessage::from(cmsg);
    packet.finalize();

    let mut buf = vec![0; packet.header.length as usize];
    packet.serialize(&mut buf[..]);
}
