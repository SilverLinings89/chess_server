use chess_library::ChessBoardState;
use rand::prelude::*;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpListener;
use std::net::TcpStream;

extern crate chess_library;

struct GameSession {
    id_white: String,
    id_black: String,
    game: ChessBoardState,
}

impl GameSession {
    pub fn new() -> Self {
        Self {
            id_black: generate_id(),
            id_white: generate_id(),
            game: ChessBoardState::new(),
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let mut sessions: Vec<GameSession> = Vec::new();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream, &mut sessions);
        println!("Connection established");
    }
}

fn generate_new_game() -> GameSession {
    GameSession::new()
}

fn generate_id() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    const ID_LENGTH: usize = 10;
    let mut rng = thread_rng();
    (0..ID_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

enum RequestType {
    newGame,
    joinWhite,
    joinBlack,
    submitMove,
    getPosition,
    none,
}

struct Request {
    request_type: RequestType,
    id: String,
    data: String,
}

impl Request {
    fn new() -> Self {
        Request {
            request_type: RequestType::none,
            data: "".to_string(),
            id: "".to_string(),
        }
    }
}

fn parse_get_request(line: String) -> Request {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts[0] != "GET" || parts.len() < 2 {
        return Request::new();
    } else {
        let route = parts[1];
        let route_parts: Vec<&str> = route.split('/').collect();
        let rt: RequestType = match route_parts[1] {
            "newGame" => RequestType::newGame,
            "joinWhite" => RequestType::joinWhite,
            "joinBlack" => RequestType::joinBlack,
            "submitMove" => RequestType::submitMove,
            "getPosition" => RequestType::getPosition,
            _ => RequestType::none,
        };
        let mut ret: Request = Request::new();
        ret.request_type = rt;
        return ret;
    }
}

fn handle_connection(mut stream: TcpStream, sessions: &mut Vec<GameSession>) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    if http_request.len() == 10 {
        let mut s: GameSession = generate_new_game();
        sessions.push(s);
    }
    let mut request_type: RequestType = RequestType::none;
    let mut param: String = String::new();
    for line in http_request {
        if line.starts_with("GET /") {
            println!("{}", line);
            if line.starts_with("GET /newGame/") {
                request_type = RequestType::newGame;
            }
            if line.starts_with("GET /joinWhite/") {
                request_type = RequestType::joinWhite;
                let start_index = "GET /joinWhite".len();
                param = line[start_index..start_index + 10].to_string();
            }
            if line.starts_with("GET /joinBlack/") {
                request_type = RequestType::joinBlack;
                let start_index = "GET /joinBlack".len();
                param = line[start_index..start_index + 10].to_string();
            }
            if line.starts_with("GET /submitMove/") {
                request_type = RequestType::submitMove;
                let start_index = "GET /submitMove/".len();
                param = line[start_index..start_index + 4].to_string();
                param = param.trim().to_string();
            }
        }
    }
    println!("Param value: {}", param);
    let status_line = "HTTP/1.1 200 OK";
    let mut contents = "".to_owned();
    match request_type {
        RequestType::newGame => {
            println!("Generating new game session...");
            let s: GameSession = generate_new_game();
            let st = format!(
                "{{ white_id:\"{}\", black_id: \"{}\"}}",
                s.id_white, s.id_black
            );
            contents.push_str(&st);
            sessions.push(s);
        }
        _ => {}
    }
    println!("At the end");
    println!("Contents: {}", contents);
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}
