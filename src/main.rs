extern crate chess_library;


fn main() {
    let mut board = ChessBoardState::new();

    println!("Hello, world! My fen ist this: {}", board.to_fen());
}
