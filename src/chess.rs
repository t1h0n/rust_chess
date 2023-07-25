#![allow(dead_code, unused_variables)]
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PieceColor {
    Black,
    White,
}

impl PieceColor {
    pub fn get_opposite(&self) -> PieceColor {
        match self {
            PieceColor::Black => PieceColor::White,
            PieceColor::White => PieceColor::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PieceType {
    King(PieceColor),
    Queen(PieceColor),
    Bishop(PieceColor),
    Knight(PieceColor),
    Rook(PieceColor),
    Pawn(PieceColor),
}

impl std::fmt::Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PieceType::King(PieceColor::White) => write!(f, "♔"),
            PieceType::King(PieceColor::Black) => write!(f, "♚"),
            PieceType::Queen(PieceColor::White) => write!(f, "♕"),
            PieceType::Queen(PieceColor::Black) => write!(f, "♛"),
            PieceType::Bishop(PieceColor::White) => write!(f, "♗"),
            PieceType::Bishop(PieceColor::Black) => write!(f, "♝"),
            PieceType::Knight(PieceColor::White) => write!(f, "♘"),
            PieceType::Knight(PieceColor::Black) => write!(f, "♞"),
            PieceType::Rook(PieceColor::White) => write!(f, "♖"),
            PieceType::Rook(PieceColor::Black) => write!(f, "♜"),
            PieceType::Pawn(PieceColor::White) => write!(f, "♙"),
            PieceType::Pawn(PieceColor::Black) => write!(f, "♟"),
        }
    }
}

impl PieceType {
    pub fn get_color(&self) -> PieceColor {
        match *self {
            PieceType::King(color)
            | PieceType::Queen(color)
            | PieceType::Bishop(color)
            | PieceType::Knight(color)
            | PieceType::Rook(color)
            | PieceType::Pawn(color) => color,
        }
    }
}
const BOARD_SIZE: std::ops::Range<i8> = 0..8;

fn is_valid_chess_position(position: Position) -> bool {
    BOARD_SIZE.contains(&position.x) && BOARD_SIZE.contains(&position.y)
}

#[derive(Debug, Clone, Copy)]
pub struct Castling {
    pub king_side: bool,
    pub queen_side: bool,
}
type Board = HashMap<Position, PieceType>;
#[derive(Debug, Clone)]
pub struct GameData {
    pub board: Board,
    pub castling: HashMap<PieceColor, Castling>,
    pub can_move_2_squares: HashSet<Position>,
    pub to_move: PieceColor,
    pub moved_2_squares: Option<Position>,
}
impl std::fmt::Display for GameData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "To Move: {:?}", self.to_move)?;
        writeln!(f, "Castling: {:?}", self.castling)?;
        writeln!(f, "En passant: {:?}", self.moved_2_squares)?;
        write!(f, " ")?;
        for i in BOARD_SIZE {
            write!(f, "{} ", i)?;
        }
        writeln!(f, " ")?;
        for y in BOARD_SIZE.rev() {
            write!(f, "{}", y)?;
            for x in BOARD_SIZE {
                if let Some(piece) = self.board.get(&Position { x, y }) {
                    write!(f, "{} ", piece)?;
                } else {
                    write!(f, "+ ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
fn place_pieces(board: &mut Board, can_move_2_squares: &mut HashSet<Position>, side: PieceColor) {
    let y = if side == PieceColor::Black { 7 } else { 0 };
    board.insert(Position { x: 0, y }, PieceType::Rook(side));
    board.insert(Position { x: 1, y }, PieceType::Knight(side));
    board.insert(Position { x: 2, y }, PieceType::Bishop(side));
    board.insert(Position { x: 3, y }, PieceType::Queen(side));
    board.insert(Position { x: 4, y }, PieceType::King(side));
    board.insert(Position { x: 5, y }, PieceType::Bishop(side));
    board.insert(Position { x: 6, y }, PieceType::Knight(side));
    board.insert(Position { x: 7, y }, PieceType::Rook(side));
    let y = if side == PieceColor::Black { 6 } else { 1 };
    for x in BOARD_SIZE {
        board.insert(Position { x, y }, PieceType::Pawn(side));
        can_move_2_squares.insert(Position { x, y });
    }
}
impl Default for GameData {
    fn default() -> Self {
        let mut board = Board::new();
        let mut can_move_2_squares = HashSet::<Position>::new();
        place_pieces(&mut board, &mut can_move_2_squares, PieceColor::White);
        place_pieces(&mut board, &mut can_move_2_squares, PieceColor::Black);
        let mut castling = HashMap::<PieceColor, Castling>::new();
        castling.insert(
            PieceColor::White,
            Castling {
                king_side: true,
                queen_side: true,
            },
        );
        castling.insert(
            PieceColor::Black,
            Castling {
                king_side: true,
                queen_side: true,
            },
        );
        GameData {
            board,
            castling,
            can_move_2_squares,
            to_move: PieceColor::White,
            moved_2_squares: None,
        }
    }
}
fn generate_en_passant_moves(game_data: &GameData, moves: &mut Moves) {
    if game_data.moved_2_squares.is_none() {
        return;
    }
    let moved_2_squares = game_data.moved_2_squares.unwrap();
    if let Some(&PieceType::Pawn(color)) = game_data.board.get(&moved_2_squares) {
        let pawns_that_might_capture = [
            Position {
                x: moved_2_squares.x - 1,
                ..moved_2_squares
            },
            Position {
                x: moved_2_squares.x + 1,
                ..moved_2_squares
            },
        ];
        let opposite = color.get_opposite();
        let y_modifier = if game_data.to_move == PieceColor::White {
            1
        } else {
            -1
        };
        for pawn_that_might_capture in pawns_that_might_capture {
            if !is_valid_chess_position(pawn_that_might_capture) {
                continue;
            }
            if let Some(&PieceType::Pawn(color)) = game_data.board.get(&pawn_that_might_capture) {
                if opposite == game_data.to_move {
                    let move_pos = Position {
                        x: moved_2_squares.x,
                        y: pawn_that_might_capture.y + y_modifier,
                    };
                    let mut new_board = game_data.board.clone();
                    new_board.remove(&moved_2_squares);
                    let moving_pawn = new_board.remove(&pawn_that_might_capture).unwrap();
                    new_board.insert(move_pos, moving_pawn);
                    if !verify_board(game_data.to_move, &new_board) {
                        continue;
                    }
                    if let Some(pawn_moves) = moves.get_mut(&pawn_that_might_capture) {
                        pawn_moves.insert(move_pos);
                    } else {
                        let mut pawn_moves = HashSet::<Position>::new();
                        pawn_moves.insert(move_pos);
                        moves.insert(pawn_that_might_capture, pawn_moves);
                    }
                }
            }
        }
    }
}

fn generate_from_points(
    position: Position,
    board: &Board,
    out: &mut HashSet<Position>,
    attack_positions: &[Position],
) {
    for &attack_position in attack_positions {
        if !is_valid_chess_position(attack_position) {
            continue;
        }

        if let Some(piece) = board.get(&attack_position) {
            if piece.get_color() == board.get(&position).unwrap().get_color() {
                continue;
            }
        }
        out.insert(attack_position);
    }
}
fn generate_generic_chunk(
    position: Position,
    board: &Board,
    out: &mut HashSet<Position>,
    generator: impl Fn(Position, i8) -> Position,
) {
    for i in BOARD_SIZE {
        let attack_pos = generator(position, i);
        if !is_valid_chess_position(attack_pos) {
            return;
        }
        if let Some(&piece) = board.get(&attack_pos) {
            if piece.get_color() != board.get(&position).unwrap().get_color() {
                out.insert(attack_pos);
            }
            return;
        } else {
            out.insert(attack_pos);
        }
    }
}
fn generate_vertical_horizontal(position: Position, board: &Board, out: &mut HashSet<Position>) {
    generate_generic_chunk(position, board, out, |pos, x| Position {
        x: pos.x - x - 1,
        ..pos
    });
    generate_generic_chunk(position, board, out, |pos, x| Position {
        x: pos.x + x + 1,
        ..pos
    });

    generate_generic_chunk(position, board, out, |pos, x| Position {
        y: pos.y + x + 1,
        ..pos
    });
    generate_generic_chunk(position, board, out, |pos, x| Position {
        y: pos.y - x - 1,
        ..pos
    });
}
fn generate_cross(position: Position, board: &Board, out: &mut HashSet<Position>) {
    generate_generic_chunk(position, board, out, |pos, x| Position {
        x: pos.x - x - 1,
        y: pos.y - x - 1,
    });
    generate_generic_chunk(position, board, out, |pos, x| Position {
        x: pos.x + x + 1,
        y: pos.y + x + 1,
    });

    generate_generic_chunk(position, board, out, |pos, x| Position {
        x: pos.x - x - 1,
        y: pos.y + x + 1,
    });
    generate_generic_chunk(position, board, out, |pos, x| Position {
        x: pos.x + x + 1,
        y: pos.y - x - 1,
    });
}

fn generate_squares_under_attack_king(
    board: &Board,
    position: Position,
    out: &mut HashSet<Position>,
) {
    let king_color = board.get(&position).unwrap().get_color();
    for i in -1i8..2 {
        for j in -1i8..2 {
            if i == 0 && j == 0 {
                continue;
            }
            let attack_position = Position {
                x: position.x + i,
                y: position.y + j,
            };
            if !is_valid_chess_position(attack_position) {
                continue;
            }
            if let Some(&piece) = board.get(&attack_position) {
                if piece.get_color() == king_color {
                    continue;
                }
            }
            out.insert(attack_position);
        }
    }
}
fn generate_squares_under_attack_queen(
    board: &Board,
    position: Position,
    out: &mut HashSet<Position>,
) {
    generate_cross(position, board, out);
    generate_vertical_horizontal(position, board, out);
}
fn generate_squares_under_attack_bishop(
    board: &Board,
    position: Position,
    out: &mut HashSet<Position>,
) {
    generate_cross(position, board, out);
}
fn generate_squares_under_attack_knight(
    board: &Board,
    position: Position,
    out: &mut HashSet<Position>,
) {
    generate_from_points(
        position,
        board,
        out,
        &[
            // x
            Position {
                x: position.x - 2,
                y: position.y + 1,
            },
            Position {
                x: position.x - 2,
                y: position.y - 1,
            },
            Position {
                x: position.x + 2,
                y: position.y + 1,
            },
            Position {
                x: position.x + 2,
                y: position.y - 1,
            },
            // y
            Position {
                x: position.x + 1,
                y: position.y - 2,
            },
            Position {
                x: position.x + 1,
                y: position.y + 2,
            },
            Position {
                x: position.x - 1,
                y: position.y - 2,
            },
            Position {
                x: position.x - 1,
                y: position.y + 2,
            },
        ],
    );
}
fn generate_squares_under_attack_rook(
    board: &Board,
    position: Position,
    out: &mut HashSet<Position>,
) {
    generate_vertical_horizontal(position, board, out);
}
fn generate_moves_pawn(game_data: &GameData, position: Position, out: &mut HashSet<Position>) {
    let (move_pos, two_squares) = match game_data.board.get(&position).unwrap().get_color() {
        PieceColor::White => (
            Position {
                y: position.y + 1,
                ..position
            },
            Position {
                y: position.y + 2,
                ..position
            },
        ),
        PieceColor::Black => (
            Position {
                y: position.y - 1,
                ..position
            },
            Position {
                y: position.y - 2,
                ..position
            },
        ),
    };
    if !game_data.board.contains_key(&move_pos) {
        out.insert(move_pos);
    }
    if game_data.can_move_2_squares.contains(&position)
        && !game_data.board.contains_key(&two_squares)
        && !game_data.board.contains_key(&move_pos)
    {
        out.insert(two_squares);
    }
    let mut attack_squares = HashSet::<Position>::new();
    generate_squares_under_attack_pawn(&game_data.board, position, &mut attack_squares);
    for attack_square in attack_squares {
        if game_data.board.contains_key(&attack_square) {
            out.insert(attack_square);
        }
    }
}
fn generate_squares_under_attack_pawn(
    board: &Board,
    position: Position,
    out: &mut HashSet<Position>,
) {
    let points = if board.get(&position).unwrap().get_color() == PieceColor::White {
        [
            Position {
                y: position.y + 1,
                x: position.x - 1,
            },
            Position {
                y: position.y + 1,
                x: position.x + 1,
            },
        ]
    } else {
        [
            Position {
                y: position.y - 1,
                x: position.x - 1,
            },
            Position {
                y: position.y - 1,
                x: position.x + 1,
            },
        ]
    };
    generate_from_points(position, board, out, &points);
}
fn generate_default_moves(game_data: &GameData, position: Position, out: &mut HashSet<Position>) {
    if let Some(&piece) = game_data.board.get(&position) {
        match piece {
            PieceType::Pawn(_) => generate_moves_pawn(game_data, position, out),
            _ => generate_squares_under_attack_for_position(&game_data.board, position, out),
        }
    }
}
fn generate_squares_under_attack_for_position(
    board: &Board,
    position: Position,
    out: &mut HashSet<Position>,
) {
    if let Some(&piece) = board.get(&position) {
        match piece {
            PieceType::King(_) => generate_squares_under_attack_king(board, position, out),
            PieceType::Queen(_) => generate_squares_under_attack_queen(board, position, out),
            PieceType::Bishop(_) => generate_squares_under_attack_bishop(board, position, out),
            PieceType::Knight(_) => generate_squares_under_attack_knight(board, position, out),
            PieceType::Rook(_) => generate_squares_under_attack_rook(board, position, out),
            PieceType::Pawn(_) => generate_squares_under_attack_pawn(board, position, out),
        }
    }
}

impl GameData {
    pub fn new() -> Self {
        Self {
            board: HashMap::new(),
            castling: HashMap::new(),
            can_move_2_squares: HashSet::new(),
            to_move: PieceColor::White,
            moved_2_squares: None,
        }
    }
}
pub fn collect_kings(board: &Board) -> HashMap<PieceColor, Position> {
    board
        .iter()
        .filter(|(_, &piece_type)| matches!(piece_type, PieceType::King(_)))
        .map(|(&position, &piece_type)| (piece_type.get_color(), position))
        .collect()
}

fn verify_board(to_move: PieceColor, new_board: &Board) -> bool {
    let king = *collect_kings(&new_board).get(&to_move).unwrap();
    let mut squares_under_attack = HashSet::<Position>::new();
    generate_squares_under_attack_for_side(
        &new_board,
        to_move.get_opposite(),
        &mut squares_under_attack,
    );
    !squares_under_attack.contains(&king)
}
fn try_make_move(game_data: &GameData, start: Position, end: Position) -> bool {
    let mut new_board = game_data.board.clone();
    let moving_piece = new_board.remove(&start).unwrap();
    new_board.insert(end, moving_piece);
    verify_board(game_data.to_move, &new_board)
}
fn generate_normal_default_moves(game_data: &GameData, moves: &mut Moves) {
    for (&piece_pos, &piece_type) in game_data.board.iter() {
        if piece_type.get_color() != game_data.to_move {
            continue;
        }
        let mut piece_moves = HashSet::<Position>::new();
        generate_default_moves(&game_data, piece_pos, &mut piece_moves);
        let mut valid_moves = HashSet::<Position>::new();
        for &piece_move in piece_moves.iter() {
            if try_make_move(&game_data, piece_pos, piece_move) {
                valid_moves.insert(piece_move);
            }
        }
        if !valid_moves.is_empty() {
            moves.insert(piece_pos, valid_moves);
        }
    }
}
fn generate_squares_under_attack_for_side(
    board: &Board,
    to_move: PieceColor,
    out: &mut HashSet<Position>,
) {
    for (&position, &piece_type) in board.iter() {
        if piece_type.get_color() == to_move {
            generate_squares_under_attack_for_position(&board, position, out);
        }
    }
}
fn castling_common(
    board: &Board,
    king_pos: Position,
    rook_pos: Position,
    final_king_pos: Position,
    final_rook_pos: Position,
    must_be_empty: &[Position],
    must_not_be_attacked: &[Position],
    attack_squares: &HashSet<Position>,
    moves: &mut Moves,
) {
    let empty_checker = |pos| board.contains_key(pos);
    let under_attack_checker = |pos| attack_squares.contains(pos);
    if must_be_empty.iter().any(empty_checker)
        || must_not_be_attacked.iter().any(under_attack_checker)
    {
        return;
    }

    if let Some(king_moves) = moves.get_mut(&king_pos) {
        king_moves.insert(final_king_pos);
    } else {
        let mut king_moves = HashSet::<Position>::new();
        king_moves.insert(final_king_pos);
        moves.insert(king_pos, king_moves);
    }
}
fn generate_castling_moves(game_data: &GameData, moves: &mut Moves) {
    let castling = game_data.castling.get(&game_data.to_move);
    if castling.is_none() {
        return;
    }
    let castling = *castling.unwrap();
    let king_pos = *collect_kings(&game_data.board)
        .get(&game_data.to_move)
        .unwrap();

    let mut attack_squares = HashSet::<Position>::new();
    generate_squares_under_attack_for_side(
        &game_data.board,
        game_data.to_move.get_opposite(),
        &mut attack_squares,
    );

    if attack_squares.contains(&king_pos) {
        return;
    }
    if castling.king_side {
        let move_path = [Position { x: 6, ..king_pos }, Position { x: 5, ..king_pos }];
        castling_common(
            &game_data.board,
            king_pos,
            Position { x: 7, ..king_pos },
            Position { x: 6, ..king_pos },
            Position { x: 5, ..king_pos },
            &move_path,
            &move_path,
            &attack_squares,
            moves,
        );
    }
    if castling.queen_side {
        let move_path = [
            Position { x: 1, ..king_pos },
            Position { x: 2, ..king_pos },
            Position { x: 3, ..king_pos },
        ];
        castling_common(
            &game_data.board,
            king_pos,
            Position { x: 7, ..king_pos },
            Position { x: 2, ..king_pos },
            Position { x: 3, ..king_pos },
            &move_path,
            &move_path[1..],
            &attack_squares,
            moves,
        );
    }
}

pub fn generate_moves(game_data: &GameData) -> Moves {
    let mut moves = Moves::new();
    generate_normal_default_moves(game_data, &mut moves);
    generate_en_passant_moves(game_data, &mut moves);
    generate_castling_moves(game_data, &mut moves);
    moves
}
pub fn postprocess_move(
    game_data: &GameData,
    start: Position,
    end: Position,
) -> (GameData, Option<Position>) {
    let mut new_game_data = game_data.clone();
    let moving_piece = new_game_data.board.remove(&start).unwrap();
    new_game_data.moved_2_squares = None;
    let mut to_be_promoted = None;
    // castling
    if matches!(moving_piece, PieceType::King(_)) {
        new_game_data.castling.remove(&game_data.to_move);
        if (start.x - end.x).abs() == 2 {
            if end.x == 6 {
                let rook = new_game_data
                    .board
                    .remove(&Position { x: 7, ..end })
                    .unwrap();
                new_game_data.board.insert(
                    Position {
                        x: end.x - 1,
                        ..end
                    },
                    rook,
                );
            } else {
                let rook = new_game_data
                    .board
                    .remove(&Position { x: 0, ..end })
                    .unwrap();
                new_game_data.board.insert(
                    Position {
                        x: end.x + 1,
                        ..end
                    },
                    rook,
                );
            }
        }
    }
    else if matches!(moving_piece, PieceType::Rook(_))
    {
        if let Some(castling) = new_game_data.castling.get_mut(&moving_piece.get_color()) {
            if start.x == 0 {
                castling.queen_side = false;
            }
            else {
                castling.king_side = false;
            }
        }
    }
    // en passant
    else if matches!(moving_piece, PieceType::Pawn(_)) {
        new_game_data.can_move_2_squares.remove(&start);
        if let Some(en_passant) = game_data.moved_2_squares {
            if en_passant.x == end.x && start.y == en_passant.y {
                new_game_data.board.remove(&en_passant);
            }
        } else if (start.y - end.y).abs() == 2 {
            new_game_data.moved_2_squares = Some(end);
        }
        if end.y == 0 || end.y == 7 {
            to_be_promoted = Some(end);
        }
    }
    new_game_data.board.insert(end, moving_piece);
    new_game_data.to_move = new_game_data.to_move.get_opposite();
    // TODO: fill with all after effects
    (new_game_data, to_be_promoted)
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Position {
    pub x: i8,
    pub y: i8,
}

pub type Moves = HashMap<Position, HashSet<Position>>;

#[test]
fn generate_en_passant_move_1_right() {
    let mut board: Board = HashMap::new();
    let to_move = PieceColor::White;
    board.insert(Position { x: 0, y: 0 }, PieceType::King(to_move));
    let moved_2_squares = Position { x: 7, y: 4 };
    board.insert(moved_2_squares, PieceType::Pawn(to_move.get_opposite()));
    let right_pos = Position { x: 6, y: 4 };
    board.insert(right_pos, PieceType::Pawn(to_move));
    let game_data = GameData {
        board,
        castling: HashMap::new(),
        can_move_2_squares: HashSet::new(),
        to_move,
        moved_2_squares: Some(moved_2_squares),
    };

    let mut moves = Moves::new();
    generate_en_passant_moves(&game_data, &mut moves);
    println!("{game_data}");
    assert_eq!(
        vec![Position { x: 7, y: 5 }],
        moves
            .get(&right_pos)
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec::<Position>>()
    );
}

#[test]
fn generate_vertical_horizontal_inclusive_test() {
    let mut out = HashSet::<Position>::new();
    let mut board = HashMap::<Position, PieceType>::new();
    board.insert(
        Position { x: 4, y: 3 },
        PieceType::Bishop(PieceColor::Black),
    );
    board.insert(
        Position { x: 3, y: 3 },
        PieceType::Bishop(PieceColor::White),
    );
    generate_vertical_horizontal(Position { x: 3, y: 3 }, &board, &mut out);
    assert!(out.contains(&Position { x: 4, y: 3 }))
}

#[test]
fn generate_vertical_horizontal_exclusive_test() {
    let mut out = HashSet::<Position>::new();
    let mut board = HashMap::<Position, PieceType>::new();
    board.insert(
        Position { x: 4, y: 3 },
        PieceType::Bishop(PieceColor::Black),
    );
    board.insert(
        Position { x: 3, y: 3 },
        PieceType::Bishop(PieceColor::Black),
    );
    generate_vertical_horizontal(Position { x: 3, y: 3 }, &board, &mut out);
    assert!(!out.contains(&Position { x: 4, y: 3 }))
}

#[test]
fn generate_vertical_horizontal_horsie_test() {
    let mut out = HashSet::<Position>::new();
    let mut board = HashMap::<Position, PieceType>::new();
    board.insert(
        Position { x: 4, y: 3 },
        PieceType::Knight(PieceColor::Black),
    );
    board.insert(
        Position { x: 6, y: 2 },
        PieceType::Bishop(PieceColor::Black),
    );
    generate_squares_under_attack_knight(&board, Position { x: 3, y: 3 }, &mut out);
    assert!(!out.contains(&Position { x: 6, y: 2 }))
}
#[test]
fn test_castling() {
    let mut moves = Moves::new();
    let mut board = HashMap::<Position, PieceType>::new();
    let king_pos = Position { x: 4, y: 7 };
    board.insert(king_pos, PieceType::King(PieceColor::Black));
    board.insert(Position { x: 7, y: 7 }, PieceType::Rook(PieceColor::Black));
    board.insert(Position { x: 0, y: 7 }, PieceType::Rook(PieceColor::Black));
    let castling_black = Castling {
        king_side: true,
        queen_side: true,
    };
    let mut castling = HashMap::<PieceColor, Castling>::new();
    castling.insert(PieceColor::Black, castling_black);
    generate_castling_moves(
        &GameData {
            board,
            castling,
            can_move_2_squares: HashSet::new(),
            to_move: PieceColor::Black,
            moved_2_squares: None,
        },
        &mut moves,
    );
    assert_eq!(moves.get(&king_pos).unwrap().len(), 2);
}

#[test]
fn test_rooks() {
    let mut board = HashMap::<Position, PieceType>::new();
    let king_pos = Position { x: 4, y: 7 };
    board.insert(king_pos, PieceType::King(PieceColor::Black));
    board.insert(Position { x: 7, y: 7 }, PieceType::Rook(PieceColor::Black));
    board.insert(Position { x: 0, y: 7 }, PieceType::Rook(PieceColor::Black));
    let castling_black = Castling {
        king_side: true,
        queen_side: true,
    };
    let mut castling = HashMap::<PieceColor, Castling>::new();
    castling.insert(PieceColor::Black, castling_black);
    let moves = generate_moves(&GameData {
        board,
        castling,
        can_move_2_squares: HashSet::new(),
        to_move: PieceColor::Black,
        moved_2_squares: None,
    });
    assert_eq!(moves.get(&Position { x: 7, y: 7 }).unwrap().len(), 9);
    assert_eq!(moves.get(&Position { x: 0, y: 7 }).unwrap().len(), 10);
}

#[test]
fn test_bishops() {
    let mut board = HashMap::<Position, PieceType>::new();
    let king_pos = Position { x: 4, y: 7 };
    board.insert(king_pos, PieceType::King(PieceColor::Black));
    board.insert(
        Position { x: 7, y: 7 },
        PieceType::Bishop(PieceColor::Black),
    );
    board.insert(
        Position { x: 0, y: 7 },
        PieceType::Bishop(PieceColor::Black),
    );
    let moves = generate_moves(&GameData {
        board,
        castling: HashMap::<PieceColor, Castling>::new(),
        can_move_2_squares: HashSet::new(),
        to_move: PieceColor::Black,
        moved_2_squares: None,
    });
    assert_eq!(moves.get(&Position { x: 7, y: 7 }).unwrap().len(), 7);
    assert_eq!(moves.get(&Position { x: 0, y: 7 }).unwrap().len(), 7);
}

#[test]
fn test_queen() {
    let mut board = HashMap::<Position, PieceType>::new();
    let king_pos = Position { x: 4, y: 7 };
    board.insert(king_pos, PieceType::King(PieceColor::Black));
    board.insert(Position { x: 4, y: 4 }, PieceType::Queen(PieceColor::Black));

    let moves = generate_moves(&GameData {
        board,
        castling: HashMap::<PieceColor, Castling>::new(),
        can_move_2_squares: HashSet::new(),
        to_move: PieceColor::Black,
        moved_2_squares: None,
    });
    assert_eq!(moves.get(&Position { x: 4, y: 4 }).unwrap().len(), 26);
}

#[test]
fn test_king_under_attack() {
    let mut board = HashMap::<Position, PieceType>::new();
    let king_pos = Position { x: 4, y: 7 };
    board.insert(king_pos, PieceType::King(PieceColor::Black));
    board.insert(Position { x: 4, y: 6 }, PieceType::Queen(PieceColor::White));

    let moves = generate_moves(&GameData {
        board,
        castling: HashMap::<PieceColor, Castling>::new(),
        can_move_2_squares: HashSet::new(),
        to_move: PieceColor::Black,
        moved_2_squares: None,
    });
    assert!(moves
        .get(&Position { x: 4, y: 7 })
        .unwrap()
        .contains(&Position { x: 4, y: 6 }));
}

#[test]
fn test_king_under_attack_unreachable() {
    let mut board = HashMap::<Position, PieceType>::new();
    let king_pos = Position { x: 4, y: 7 };
    board.insert(king_pos, PieceType::King(PieceColor::Black));
    board.insert(Position { x: 3, y: 5 }, PieceType::Queen(PieceColor::White));

    let moves = generate_moves(&GameData {
        board,
        castling: HashMap::<PieceColor, Castling>::new(),
        can_move_2_squares: HashSet::new(),
        to_move: PieceColor::Black,
        moved_2_squares: None,
    });
    assert!(!moves
        .get(&Position { x: 4, y: 7 })
        .unwrap()
        .contains(&Position { x: 4, y: 6 }));
}
