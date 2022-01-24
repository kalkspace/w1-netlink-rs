use netlink_packet_core::{NetlinkMessage, NLM_F_ACK, NLM_F_REQUEST};
use netlink_sys::{protocols::NETLINK_CONNECTOR, Socket, SocketAddr};
use w1_netlink::proto::{
    connector::NlConnectorMessage,
    message::{MasterId, TargetId, W1MessageType, W1NetlinkMessage},
};

fn main() {
    let msg =
        W1NetlinkMessage::<MasterId>::new(W1MessageType::ListMasters, TargetId::master_id(0), []);
    let cmsg = NlConnectorMessage::new(0, [msg]);

    let mut nl_msg = NetlinkMessage::from(cmsg);
    nl_msg.header.port_number = std::process::id();
    nl_msg.header.flags = NLM_F_ACK | NLM_F_REQUEST;
    nl_msg.finalize();

    let buf_len = nl_msg.buffer_len();
    let mut msg = vec![0; buf_len];
    nl_msg.serialize(&mut msg[..]);

    //println!("msg to send: {:#04X?}", msg);

    let mut socket = Socket::new(NETLINK_CONNECTOR).unwrap();
    let _ = socket.bind_auto().unwrap();
    println!("bound.");

    let kernel_addr = SocketAddr::new(0, 0);
    socket.connect(&kernel_addr).unwrap();
    println!("connected.");

    let n_sent = socket.send(&msg[..], 0).unwrap();
    assert_eq!(n_sent, msg.len());
    println!("sent.");

    // buffer for receiving the response
    let mut buf = vec![0; 4096];
    loop {
        let n_received = socket.recv(&mut &mut buf[..], 0).unwrap();
        println!("received {:#04X?}", &buf[..n_received]);
        let resp = NetlinkMessage::<NlConnectorMessage<W1NetlinkMessage<MasterId>>>::deserialize(
            &buf[0..n_received],
        )
        .unwrap();
        println!("resp: {:?}", resp);
        if buf[4] == 2 && buf[5] == 0 {
            println!("the kernel responded with an error");
            return;
        }
        if buf[4] == 3 && buf[5] == 0 {
            println!("end of dump");
            return;
        }
    }
}
