use clap::{ArgEnum, Parser};
use std::fmt;
use rand::Rng;
use std::net::UdpSocket;
//use std::net::{IpAddr,Ipv4Addr,Ipv6Addr};
//use std::ops::RangeInclusive;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum QType { // not exhaustive. also used to map to decimal values for pkts.
    A = 1,
    AAAA = 28,
    NS = 2,
    MX = 15,
    SVCB = 64,
    HTTPS = 65,
    CNAME = 5,
    TXT = 16,
    ANY = 255
}

/* unneeded, since enums can have values associated with each variant (above) - see
   https://stackoverflow.com/questions/36928569/how-can-i-create-enums-with-constant-values-in-rust */
/*
fn qtype_to_u16(t: QType) -> u16 {
    match t {
        QType::A => 1,
        QType::AAAA => 28,
        QType::NS => 2,
        QType::MX => 15,
        QType::SVCB => 64,
        QType::HTTPS => 65,
        QType::CNAME => 5,
        QType::TXT => 16,
        QType::ANY => 255
    }
}
*/

// with or without this, the argenum lowercases the enum variants in the 
// help text. {:?} prints the type as uppercase... but we want the argparser to use that too!
impl fmt::Display for QType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QType::A => write!(f, "A"),
            QType::AAAA => write!(f, "AAAA"),
            QType::NS => write!(f, "NS"),
            QType::MX => write!(f, "MX"),
            QType::SVCB => write!(f, "SVCB"),
            QType::HTTPS => write!(f, "HTTPS"),
            QType::CNAME => write!(f, "CNAME"),
            QType::TXT => write!(f, "TXT"),
            QType::ANY => write!(f, "ANY"),
        }
    }
}

/* The idea was to use "idiomatic" rust to store the server address - but that was
   before I knew that the socket creation functions didn't require an ip-like object,
   and were happy with being passed "foo.bar.com:53" or "8.8.8.8:53". because of this,
   I removed the over-engineered code below, and kept server address as a simple string. */
/*
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ServerAddress {
    Hostname(String),
    IP(IpAddr)
}

impl fmt::Display for ServerAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServerAddress::Hostname(s) => write!(f, "{}", s),
            ServerAddress::IP(ip) => write!(f, "{}", ip)
        }
    }
}

fn addr_parser(s: &str) -> Result<ServerAddress, String> {
    /* I tried doing something like this:
       if let ip = s.parse::<Ipv4Addr>() { ... }
       but the compiler said that the pattern would always match,
       which I don't understand - how would it match if s was "foo.bar.com"? */

    let ipv4 = s.parse::<Ipv4Addr>();
    if ipv4.is_ok() {
        return Ok(ServerAddress::IP(IpAddr::V4(ipv4.unwrap())))
    } 
    
    let ipv6 = s.parse::<Ipv6Addr>();
    if ipv6.is_ok() {
        return Ok(ServerAddress::IP(IpAddr::V6(ipv6.unwrap())))
    }
    
    /* if we get here, either we have a hostname, or parsing of the ip addr failed.
       what should be done if the ip addr parse failed (say, on 999.0.0.0)? */
    Ok(ServerAddress::Hostname(String::from(s)))
    /* TODO and what if the hostname is garbage? 
       Err(String::from("server name is garbage")); */
}
*/

// taken from https://github.com/clap-rs/clap/blob/v3.2.5/examples/tutorial_derive/04_02_validate.rs
// from earlier attempts to split out addr/port as diff arguments
/*
const PORT_RANGE: RangeInclusive<usize> = 1..=65535;
fn port_parser(s: &str) -> Result<u16, String> {
    let port: usize = s.parse().map_err(|_| format!("{} isn't an integer!", s))?;
    if PORT_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!("Port not in range {}-{}", PORT_RANGE.start(), PORT_RANGE.end()))
    }
}
*/

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Arguments {
    // see notes above on using an enum instead of just a string here.
    /*
    #[clap(short='s', long, value_parser=addr_parser, default_value_t = ServerAddress::IP(IpAddr::V4(Ipv4Addr::new(8,8,8,8))))]
    server: ServerAddress,
    */
    
    #[clap(short='s', long, value_parser, default_value_t = String::from("8.8.8.8:53"))]
    server: String,

    // previously, I split out the server addr and port; it's simpler to put both
    // in the server field instead.
    /*
    #[clap(short='p', long, value_parser=port_parser, default_value_t = 53u16)]
    port: u16,
    */

    /* TODO validation? */
    #[clap(short='n', long, value_parser, default_value_t = String::from("google.com"))]
    qname: String,
    
    // TODO figure out why this lowercases all the QType members in the -h text, even with the 
    // "impl fmt::Display for Qtype" above
    // TODO figure out why '--type A' doesn't work but '--type a' does.
    // possibly related to https://github.com/clap-rs/clap/issues/891 ?
    #[clap(short='t', long, value_parser, arg_enum, default_value_t = QType::A)]
    qtype: QType,
}

fn make_qname_string(qname: &String) -> Vec<u8> {
    let mut ret : Vec<u8> = Vec::new();
    let mut qname_copy = qname.clone();
    if qname.ends_with('.') {
        qname_copy.pop();
    }
    let split : Vec<&str> = qname_copy.split('.').collect();
    for s in split {
        // first, push the label length
        ret.push(s.len() as u8);
        // then, push the label itself
        ret.extend_from_slice(s.as_bytes());
    }
    ret.push(0u8); // null byte after labels
    ret
}

fn make_query(args: &Arguments) -> Vec<u8> {
    let mut ret : Vec<u8> = Vec::new();
    let mut rng = rand::thread_rng();
    // layout from https://routley.io/posts/hand-writing-dns-messages/


    // header
    let qid = rng.gen::<u16>();
    ret.extend_from_slice(&qid.to_be_bytes());
    let options = 0b0000_0001_0010_0000u16;
    ret.extend_from_slice(&options.to_be_bytes());
    let qcount = 1u16;
    ret.extend_from_slice(&qcount.to_be_bytes());
    let other_count = 0u16; // an/ns/ar count
    ret.extend_from_slice(&other_count.to_be_bytes());
    ret.extend_from_slice(&other_count.to_be_bytes());
    ret.extend_from_slice(&other_count.to_be_bytes());

    // question
    let mut qname_str = make_qname_string(&args.qname);
    ret.append(&mut qname_str);
    let qtype = args.qtype as u16;
    ret.extend_from_slice(&qtype.to_be_bytes());
    let qclass = 1u16;
    ret.extend_from_slice(&qclass.to_be_bytes());

    ret
}

struct QuestionRecord {
    name: String,
    qtype: QType,
    class: u16
}

struct ResourceRecord {
    name: String,
    rtype: QType,
    class: u16,
    ttl: u32,
    rdata: Vec<u8> // TODO make this into an enum based on the type of the data.
}

struct ResponsePacketStruct {
    qid: u16,
    flags: u16,
    questions: Vec<QuestionRecord>,
    answers: Vec<ResourceRecord>,
    auths: Vec<ResourceRecord>,
    additionals: Vec<ResourceRecord>
}

fn parse_question_records(buf: &[u8], count: u16, q: &mut Vec<QuestionRecord>) -> Result<u32, String> {
    Ok(0) // TODO implement
}

fn parse_answer_records(buf: &[u8], count: u16, a: &mut Vec<QuestionRecord>) -> Result<u32, String> {
    Ok(0) // TODO implement
}

fn parse_auth_records(buf: &[u8], count: u16, a: &mut Vec<QuestionRecord>) -> Result<u32, String> {
    Ok(0) // TODO implement
}

fn parse_additional_records(buf: &[u8], count: u16, a: &mut Vec<QuestionRecord>) -> Result<u32, String> {
    Ok(0) // TODO implement
}

fn parse_response_packet(buf: &[u8]) -> Result<ResponsePacketStruct, String> {
    if buf.len() < 12 {
        return Err(String::from("buf length less than 12 bytes - cannot parse"));
    }

    let mut questions: Vec<QuestionRecord> = Vec::new();
    let mut answers: Vec<ResourceRecord> = Vec::new();
    let mut auths: Vec<ResourceRecord> = Vec::new();
    let mut additionals: Vec<ResourceRecord> = Vec::new();

    let mut twobytes = [0u8, 0u8];
    twobytes.clone_from_slice(&buf[0 .. 2]);
    let qid = u16::from_be_bytes(twobytes);
    twobytes.clone_from_slice(&buf[2 .. 4]);
    let flags = u16::from_be_bytes(twobytes);
    twobytes.clone_from_slice(&buf[4 .. 6]);
    let qcount = u16::from_be_bytes(twobytes);
    twobytes.clone_from_slice(&buf[6 .. 8]);
    let ancount = u16::from_be_bytes(twobytes);
    twobytes.clone_from_slice(&buf[8 .. 10]);
    let authcount = u16::from_be_bytes(twobytes);
    twobytes.clone_from_slice(&buf[10 .. 12]);
    let addcount = u16::from_be_bytes(twobytes);

    if buf.len() == 12 {
        // if end of buf right after header, ignore any counts we read - 
        // they don't exist in the buffer. set those counts to 0 in the returned var
        // and return immediately.
        return Ok(ResponsePacketStruct { 
            qid:qid, flags:flags,
            questions:questions, answers:answers, 
            auths:auths, additionals:additionals});
    }

    let mut offset: usize = 12; // records (question and resources) follow
    match parse_question_records(&buf[offset..], qcount, &mut questions) {
        Ok(count) => { offset += count as usize },
        Err(s) => return Err(s)
    }
    match parse_answer_records(&buf[offset..], ancount, &mut answers) {
        Ok(count) => { offset += count as usize },
        Err(s) => return Err(s)
    }
    match parse_auth_records(&buf[offset..], authcount, &mut auths) {
        Ok(count) => { offset += count as usize },
        Err(s) => return Err(s)
    }
    match parse_additional_records(&buf[offset..], addcount, &mut additionals) {
        Ok(count) => { offset += count as usize },
        Err(s) => return Err(s)
    }

    Ok(ResponsePacketStruct { 
        qid: qid, flags: flags, 
        questions: questions, answers: answers,
        auths: auths, additionals: additionals})
}

fn print_response(r: ResponsePacketStruct) {
    println!("Got a response:");
    println!("  QID = {:#X}", r.qid); // uppercase hex with preceding 0x
    println!("  flags = {:b}", r.flags); // binary w/out preceding 0b
    println!("  qcount = {}", r.questions.len());
    println!("  ancount = {}", r.answers.len());
    println!("  authcount = {}", r.auths.len());
    println!("  addcount = {}", r.additionals.len());
    // TODO output records
}

fn main() {
    let args = Arguments::parse();
    println!("Hitting {} with a/an {} query for {}", args.server, args.qtype, args.qname);
 
    let qpkt = make_query(&args);
    // TODO get addr/random port to bind to programatically rather than hard-coding it here.
    let socket = UdpSocket::bind("192.168.1.16:43254").expect("couldn't bind to address");
    socket.connect(args.server).expect("couldn't connect to server");
    match socket.send(qpkt.as_slice()) {
        Ok(send_size) => {
            println!("successfully sent {} bytes", send_size);
            let mut rbuf = [0; 65535];
            match socket.recv(&mut rbuf) {
                Ok(response_length) => {
                    println!("Got {} response bytes back", response_length); 
                    let response_packet = &rbuf[0 .. response_length];
                    let response_struct = parse_response_packet(response_packet);
                    match response_struct {
                        Ok(resp) => print_response(resp),
                        Err(s) => println!("Failed to parse response packet: {}", s)
                    }
                },
                Err(s) => println!("Error reading response from server: {}", s)
            }
        },
        Err(s) => println!("Error sending to socket: {}", s)
    }
}
