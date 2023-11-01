use std::{
    io::Write,
    net::{IpAddr, TcpStream},
    str::FromStr,
    sync::{Arc, Mutex},
};

use const_format::formatcp;
use easy_repl::{command, CommandStatus, Repl};

use routecore::{asn::Asn, bmp::message::PeerType};
use routes::bmp::encode::{
    mk_initiation_msg, mk_peer_down_notification_msg,
    mk_peer_up_notification_msg, mk_raw_route_monitoring_msg,
    mk_route_monitoring_msg, mk_termination_msg, Announcements, MyPeerType,
    PerPeerHeader, Prefixes,
};

const DEF_BMP_PORT: u16 = 11019;

/// initiation a b
/// peer_up_notification global 0 10.0.0.1 12345 127.0.0.1 80 81 888 999 0 0
/// route_monitoring global 0 10.0.0.1 12345 0 127.0.0.1/32
/// route_monitoring global 0 10.0.0.1 12345 0 none "e [123,456,789] 10.0.0.1 none 127.0.0.1/32"
/// route_monitoring global 0 10.0.0.1 12345 0 none "e [123,456,789] 10.0.0.1 BLACKHOLE,123:44 127.0.0.1/32"
fn main() {
    let server_arg = clap::Arg::new("server")
        .short('s')
        .long("server")
        .required(true)
        .value_name("IP or IP:PORT")
        .help(formatcp!("Connect to a BMP monitoring station on this address [default port: {DEF_BMP_PORT}]"));

    let matches = clap::Command::new("bmp-speaker")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .arg(server_arg)
        .get_matches();

    let server = matches.get_one::<String>("server").unwrap();

    let server = if !server.contains(":") {
        format!("{server}:{DEF_BMP_PORT}")
    } else {
        server.to_string()
    };

    match TcpStream::connect(&server) {
        Err(err) => {
            eprintln!(
                "Error: Failed to connect to server at '{}': {}",
                server, err
            );
        }

        Ok(stream) => {
            let stream = Arc::new(Mutex::new(stream));

            let mut repl = Repl::builder()
                .add("initiation", initiate_cmd(stream.clone()))
                .add("peer_up_notification", peer_up_cmd(stream.clone()))
                .add("route_monitoring", route_monitoring_cmd(stream.clone()))
                .add(
                    "raw_route_monitoring",
                    route_monitoring_raw_cmd(stream.clone()),
                )
                .add("peer_down_notification", peer_down_cmd(stream.clone()))
                .add("termination", terminate_cmd(stream))
                .build()
                .expect("Failed to create REPL");

            repl.run().expect("Critical REPL error");
        }
    }
}

fn initiate_cmd<'a>(stream: Arc<Mutex<TcpStream>>) -> easy_repl::Command<'a> {
    command! {
        "BMP Initiation Message",
        (sys_name: String, sys_descr: String) => |sys_name: String, sys_descr: String| {
            stream.lock().unwrap().write_all(mk_initiation_msg(&sys_name, &sys_descr).as_ref()).unwrap();
            Ok(CommandStatus::Done)
        }
    }
}

fn peer_up_cmd<'a>(stream: Arc<Mutex<TcpStream>>) -> easy_repl::Command<'a> {
    command! {
        "BMP Peer Up Notification",
        (
            peer_type: MyPeerType,
            peer_flags: u8,
            peer_address: IpAddr,
            peer_as: Asn,
            local_address: IpAddr,
            local_port: u16,
            remote_port: u16,
            sent_open_asn: u16,
            received_open_asn: u16,
            sent_bgp_identifier: u32,
            received_bgp_identifier: u32
        ) => |
            peer_type: MyPeerType,
            peer_flags: u8,
            peer_address: IpAddr,
            peer_as: Asn,
            local_address: IpAddr,
            local_port: u16,
            remote_port: u16,
            sent_open_asn: u16,
            received_open_asn: u16,
            sent_bgp_identifier: u32,
            received_bgp_id: u32
        | {
            let peer_distinguisher = match *peer_type {
                PeerType::GlobalInstance => [0u8; 8],
                _ => todo!(),
            };
            let peer_bgp_id = received_bgp_id.to_be_bytes();
            let per_peer_header = PerPeerHeader {
                peer_type,
                peer_flags,
                peer_distinguisher,
                peer_address,
                peer_as,
                peer_bgp_id};
            let (bytes, warnings) = mk_peer_up_notification_msg(
                &per_peer_header,
                local_address,
                local_port,
                remote_port,
                sent_open_asn,
                received_open_asn,
                sent_bgp_identifier,
                received_bgp_id,
                vec![],
                true);
            for msg in warnings {
                eprintln!("Warning: {}", msg);
            }
            stream.lock().unwrap().write_all(
                bytes
                .as_ref())
                .unwrap();
            Ok(CommandStatus::Done)
        }
    }
}

fn route_monitoring_cmd<'a>(
    stream: Arc<Mutex<TcpStream>>,
) -> easy_repl::Command<'a> {
    command! {
        "BMP Route Monitoring Message (from announcements & withdrawals)",
        (
            peer_type: MyPeerType,
            peer_flags: u8,
            peer_address: IpAddr,
            peer_as: Asn,
            peer_bgp_id: u32,
            withdrawals: Prefixes,
            announcements: Announcements
        ) => |
            peer_type: MyPeerType,
            peer_flags: u8,
            peer_address: IpAddr,
            peer_as: Asn,
            peer_bgp_id: u32,
            withdrawals: Prefixes,
            announcements: Announcements,
        | {
            let peer_distinguisher = match *peer_type {
                PeerType::GlobalInstance => [0u8; 8],
                _ => todo!(),
            };
            let peer_bgp_id = peer_bgp_id.to_be_bytes();
            let per_peer_header = PerPeerHeader {
                peer_type,
                peer_flags,
                peer_distinguisher,
                peer_address,
                peer_as,
                peer_bgp_id};
            let (bytes, warnings) = mk_route_monitoring_msg(&per_peer_header, &withdrawals, &announcements, &[]);
            for msg in warnings {
                eprintln!("Warning: {}", msg);
            }
            stream.lock().unwrap().write_all(bytes.as_ref()).unwrap();
            Ok(CommandStatus::Done)
        }
    }
}

struct HexBytes(Vec<u8>);

impl HexBytes {
    pub fn to_vec(self) -> Vec<u8> {
        self.0
    }
}

impl FromStr for HexBytes {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = Vec::new();
        let hex_codes = s.split(|c| c == ' ' || c == ',').collect::<Vec<_>>();
        for hex_code in hex_codes {
            let hex_digits = hex_code.strip_prefix("0x").unwrap_or(hex_code);
            let n = u16::from_str_radix(hex_digits, 16)?;
            out.extend_from_slice(&n.to_be_bytes());
        }
        Ok(Self(out))
    }
}

fn route_monitoring_raw_cmd<'a>(
    stream: Arc<Mutex<TcpStream>>,
) -> easy_repl::Command<'a> {
    command! {
        "BMP Route Monitoring Message (from hex BGP UPDATE bytes)",
        (
            peer_type: MyPeerType,
            peer_flags: u8,
            peer_address: IpAddr,
            peer_as: Asn,
            peer_bgp_id: u32,
            bgp_msg_buf: HexBytes
        ) => |
            peer_type: MyPeerType,
            peer_flags: u8,
            peer_address: IpAddr,
            peer_as: Asn,
            peer_bgp_id: u32,
            bgp_msg_buf: HexBytes,
        | {
            let peer_distinguisher = match *peer_type {
                PeerType::GlobalInstance => [0u8; 8],
                PeerType::RdInstance => todo!(),
                PeerType::LocalInstance => todo!(),
                _ => todo!(),
            };
            let peer_bgp_id = peer_bgp_id.to_be_bytes();
            let per_peer_header = PerPeerHeader {
                peer_type,
                peer_flags,
                peer_distinguisher,
                peer_address,
                peer_as,
                peer_bgp_id};
            let (bytes, warnings) = mk_raw_route_monitoring_msg(&per_peer_header, bgp_msg_buf.to_vec().into());
            for msg in warnings {
                eprintln!("Warning: {}", msg);
            }
            stream.lock().unwrap().write_all(bytes.as_ref()).unwrap();
            Ok(CommandStatus::Done)
        }
    }
}

fn peer_down_cmd<'a>(
    stream: Arc<Mutex<TcpStream>>,
) -> easy_repl::Command<'a> {
    command! {
        "BMP Peer Down Notification",
        (
            peer_type: MyPeerType,
            peer_flags: u8,
            peer_address: IpAddr,
            peer_as: Asn,
            peer_bgp_id: u32
        ) => |
            peer_type: MyPeerType,
            peer_flags: u8,
            peer_address: IpAddr,
            peer_as: Asn,
            peer_bgp_id: u32,
        | {
            let peer_distinguisher = match *peer_type {
                PeerType::GlobalInstance => [0u8; 8],
                _ => todo!(),
            };
            let peer_bgp_id = peer_bgp_id.to_be_bytes();
            let per_peer_header = PerPeerHeader {
                peer_type,
                peer_flags,
                peer_distinguisher,
                peer_address,
                peer_as,
                peer_bgp_id};
            let (bytes, warnings) = mk_peer_down_notification_msg(&per_peer_header);
            for msg in warnings {
                eprintln!("Warning: {}", msg);
            }
            stream.lock().unwrap().write_all(bytes.as_ref()).unwrap();
            Ok(CommandStatus::Done)
        }
    }
}

fn terminate_cmd<'a>(
    stream: Arc<Mutex<TcpStream>>,
) -> easy_repl::Command<'a> {
    command! {
        "BMP Termination Message", () => || {
            stream.lock().unwrap().write_all(mk_termination_msg().as_ref()).unwrap();
            Ok(CommandStatus::Done)
        }
    }
}
