use std::collections::HashMap;
use std::fmt;

use ggez::{
    event::{EventHandler, MouseButton},
    graphics::{self, Color, DrawMode, DrawParam, Image},
    Context, GameResult,
};

use std::vec::Vec;

const SQUARE_SIZE: i32 = 100;


#[derive(Clone, Copy)]
enum Player {
    White,
    Black,
}

impl Player {
    fn switch(&self) -> Self {
        match *self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

struct Point<T>
where
    T: Copy,
{
    x: T,
    y: T,
}

impl<T> Point<T>
where
    T: Copy,
{
    fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> std::clone::Clone for Point<T>
where
    T: Copy,
{
    fn clone(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
        }
    }
}

struct BoardState {
    board: [[char; 8]; 8],
    player: Player,
    wk_pos: (u8, u8),
    bk_pos: (u8, u8),
    enp_b: u8,
    enp_w: u8,
    castling: u8,
    b_check: bool,
    w_check: bool,
    b_checks: u8,
    w_checks: u8,
    w_win: bool,
    b_win: bool,
}

impl std::clone::Clone for BoardState {
    fn clone(&self) -> Self {
        Self {
            board: self.board.clone(),
            player: self.player,
            wk_pos: self.wk_pos,
            bk_pos: self.bk_pos,
            enp_b: self.enp_b,
            enp_w: self.enp_w,
            castling: self.castling,
            b_check: self.b_check,
            w_check: self.w_check,
            b_checks: self.b_checks,
            w_checks: self.w_checks,
            w_win: self.w_win,
            b_win: self.b_win,
        }
    }
}

impl fmt::Display for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = String::from("[\n");

        for row in self.board.iter() {
            out.push_str(&format!("\t{:?}\n", row));
        }

        out.push_str("]");

        write!(f, "{}", out)
    }
}

pub struct RChess {
    board: [[Color; 8]; 8],
    board_pcs: [[char; 8]; 8],
    current: Option<char>,
    current_pos: Option<(u8, u8)>,
    moves: Vec<(u8, u8)>,
    pieces: HashMap<char, Image>,
    turn: Player,
    w_color: Color,
    b_color: Color,
    sq_size: i32,
    moving: bool,
    needs_draw: bool,
    enp_b: u8,
    enp_w: u8,
    w_king_pos: (u8, u8),
    b_king_pos: (u8, u8),
    castling: u8,
    w_check: bool,
    b_check: bool,
    b_checks: u8,
    w_checks: u8,
    w_win: bool,
    b_win: bool,
}

impl RChess {
    // Create a new instance of RChess
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut pieces = HashMap::<char, Image>::new();

        let board_pcs = [
            ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'],
            ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
            ['-', '-', '-', '-', '-', '-', '-', '-'],
            ['-', '-', '-', '-', '-', '-', '-', '-'],
            ['-', '-', '-', '-', '-', '-', '-', '-'],
            ['-', '-', '-', '-', '-', '-', '-', '-'],
            ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
            ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'],
        ];

        for row in board_pcs.iter() {
            for piece in row.iter() {
                if pieces.contains_key(piece) {
                    continue;
                }

                if piece != &'-' {
                    let img = Image::new(ctx, &format!("/{}.png", piece))?;
                    pieces.insert(*piece, img);
                }
            }
        }

        let w_color = Color::from_rgb(222, 222, 222);
        let b_color = Color::from_rgb(40, 40, 40);

        let mut chess = Self {
            board: [[w_color.clone(); 8]; 8],
            board_pcs,
            current: None,
            current_pos: None,
            moves: Vec::new(),
            pieces: pieces,
            turn: Player::White,
            w_color,
            b_color,
            sq_size: SQUARE_SIZE,
            moving: false,
            needs_draw: true,
            enp_b: 0,
            enp_w: 0,
            w_king_pos: (4, 7),
            b_king_pos: (4, 0),
            castling: 0b1111,
            w_check: false,
            b_check: false,
            b_checks: 0,
            w_checks: 0,
            w_win: false,
            b_win: false,
        };

        chess.reset_board();

        Ok(chess)
    }
    
    fn reset_board(&mut self) {
        for y in 0..8 {
            let row_even = y % 2 == 0;

            for x in 0..8 {
                let col_even = x % 2 == 0;

                self.board[y][x] = if (col_even && row_even) || (!col_even && !row_even) {
                    self.w_color.clone()
                } else {
                    self.b_color.clone()
                }
            }
        }
        let center_w_color = Color::from_rgb(255, 215, 0);
        let center_b_color = Color::from_rgb(185, 145, 0);
        self.board[3][4] = center_b_color;
        self.board[4][3] = center_b_color;
        self.board[3][3] = center_w_color;
        self.board[4][4] = center_w_color;
    }

    fn get_board_state(&self) -> BoardState {
        BoardState {
            board: self.board_pcs.clone(),
            player: self.turn,
            wk_pos: self.w_king_pos,
            bk_pos: self.b_king_pos,
            castling: self.castling,
            enp_b: self.enp_b,
            enp_w: self.enp_w,
            w_check: self.w_check,
            b_check: self.b_check,
            b_checks: self.b_checks,
            w_checks: self.w_checks,
            w_win: self.w_win,
            b_win: self.b_win,
        }
    }

    fn is_white_piece(pc: char) -> bool {
        ['K', 'Q', 'R', 'N', 'B', 'P'].contains(&pc)
    }

    fn is_black_piece(pc: char) -> bool {
        ['k', 'q', 'r', 'n', 'b', 'p'].contains(&pc)
    }

    fn is_opponent(plyr: Player, ch: char) -> bool {
        match plyr {
            Player::White => Self::is_black_piece(ch),
            Player::Black => Self::is_white_piece(ch),
        }
    }

    fn is_piece(ch: char) -> bool {
        ch != '-'
    }
    
    fn white_won(&mut self) -> () {
        self.w_win = true;
    }

    fn black_won(&mut self) {
        self.b_win = true;
    }

    fn move_piece_to(from: Point<u8>, to: Point<u8>, state: &mut BoardState) {
        let x = from.x as usize;
        let y = from.y as usize;

        let ch = state.board[y][x];
        state.enp_b = 0;
        state.enp_w = 0;

        match ch {
            'K' => {
                state.wk_pos = (to.x, to.y);
                state.castling &= 0b0011;

                if (from.x, from.y) == (4, 7) {
                    if (to.x, to.y) == (6, 7) {
                        state.board[7][5] = 'R';
                        state.board[7][7] = '-';
                    } else if (to.x, to.y) == (2, 7) {
                        state.board[7][3] = 'R';
                        state.board[7][0] = '-';
                    }
                }
            }

            'k' => {
                state.bk_pos = (to.x, to.y);
                state.castling &= 0b1100;

                if (from.x, from.y) == (4, 0) {
                    if (to.x, to.y) == (6, 0) {
                        state.board[0][5] = 'r';
                        state.board[0][7] = '-';
                    } else if (to.x, to.y) == (2, 0) {
                        state.board[0][3] = 'r';
                        state.board[0][0] = '-';
                    }
                }
            }

            'p' => {
                if from.y == 1 && to.y == 3 {
                    state.enp_b = 0x80 >> from.x;
                } else if from.y == 4 && from.x != to.x && state.board[5][to.x as usize] == '-' {
                    state.board[4][to.x as usize] = '-';
                }
            }

            'P' => {
                if from.y == 6 && to.y == 4 {
                    state.enp_w = 0x80 >> from.x;
                } else if from.y == 3 && from.x != to.x && state.board[2][to.x as usize] == '-' {
                    state.board[3][to.x as usize] = '-';
                }
            }

            'r' => {
                if from.x == 0 && from.y == 0 {
                    state.castling &= 0b1101;
                } else if from.x == 7 && from.y == 0 {
                    state.castling &= 0b1110;
                }
            }

            'R' => {
                if from.x == 0 && from.y == 7 {
                    state.castling &= 0b0111;
                } else if from.x == 7 && from.y == 7 {
                    state.castling &= 0b1011;
                }
            }

            _ => (),
        }

        state.board[to.y as usize][to.x as usize] = ch;
        state.board[y][x] = '-';

        state.b_check = Self::check_for_checks(Player::Black, state);
        state.w_check = Self::check_for_checks(Player::White, state);
        if state.w_check { state.b_checks = state.b_checks+1; }
        if state.b_check { state.w_checks = state.w_checks+1; }
    }

    fn get_line_moves(pos: &Point<u8>, dpos: Point<i8>, state: &BoardState) -> Vec<(u8, u8)> {
        let mut m_x = pos.x as i8 + dpos.x;
        let mut m_y = pos.y as i8 + dpos.y;

        let mut moves = Vec::<(u8, u8)>::with_capacity(7);

        while m_x >= 0 && m_x < 8 && m_y >= 0 && m_y < 8 {
            let ch = state.board[m_y as usize][m_x as usize];

            if Self::is_piece(ch) {
                if Self::is_opponent(state.player, ch) {
                    moves.push((m_x as u8, m_y as u8));
                }

                break;
            }

            moves.push((m_x as u8, m_y as u8));
            m_x += dpos.x;
            m_y += dpos.y;
        }

        moves
    }

    fn mv_pawn(pos: Point<u8>, state: &BoardState) -> Vec<(u8, u8)> {
        let x_i = pos.x as usize;
        let y_i = pos.y as usize;

        let mut moves = Vec::<(u8, u8)>::with_capacity(4);

        match state.player {
            Player::White => {
                if pos.y == 0 {
                    return moves;
                }

                if !Self::is_piece(state.board[y_i - 1][x_i]) {
                    moves.push((pos.x, pos.y - 1));
                }

                if pos.y == 6 && !Self::is_piece(state.board[y_i - 2][x_i]) {
                    moves.push((pos.x, pos.y - 2));
                }

                if (pos.x < 7 && Self::is_opponent(state.player, state.board[y_i - 1][x_i + 1]))
                    || (pos.y == 3 && pos.x < 7 && state.enp_b & (0x80 >> (pos.x + 1)) > 0)
                {
                    moves.push((pos.x + 1, pos.y - 1));
                }

                if (pos.x > 0 && Self::is_opponent(state.player, state.board[y_i - 1][x_i - 1]))
                    || (pos.y == 3 && pos.x > 0 && state.enp_b & (0x80 >> (pos.x - 1)) > 0)
                {
                    moves.push((pos.x - 1, pos.y - 1));
                }
            }

            Player::Black => {
                if pos.y == 7 {
                    return moves;
                }

                if !Self::is_piece(state.board[y_i + 1][x_i]) {
                    moves.push((pos.x, pos.y + 1));
                }

                if pos.y == 1 && !Self::is_piece(state.board[y_i + 2][x_i]) {
                    moves.push((pos.x, pos.y + 2));
                }

                if (pos.x < 7 && Self::is_opponent(state.player, state.board[y_i + 1][x_i + 1]))
                    || (pos.y == 4 && pos.x < 7 && state.enp_w & (0x80 >> (pos.x + 1)) > 0)
                {
                    moves.push((pos.x + 1, pos.y + 1));
                }

                if (pos.x > 0 && Self::is_opponent(state.player, state.board[y_i + 1][x_i - 1]))
                    || (pos.y == 4 && pos.x > 0 && state.enp_w & (0x80 >> (pos.x - 1)) > 0)
                {
                    moves.push((pos.x - 1, pos.y + 1));
                }
            }
        }

        moves
    }

    fn mv_knight(pos: Point<u8>, state: &BoardState) -> Vec<(u8, u8)> {
        let x_m = pos.x as i8;
        let y_m = pos.y as i8;

        let moves: Vec<(i8, i8)> = vec![
            (-2, -1),
            (-1, -2),
            ( 1, -2),
            ( 2, -1),
            (-2,  1),
            (-1,  2),
            ( 1,  2),
            ( 2,  1),
        ];

        let mut poss_moves = Vec::<(u8, u8)>::with_capacity(8);

        for (dx, dy) in moves {
            let pos_x = x_m + dx;
            let pos_y = y_m + dy;
            if pos_x >= 0 && pos_x < 8 && pos_y >= 0 && pos_y < 8 {
                let ch = state.board[pos_y as usize][pos_x as usize];
                if !Self::is_piece(ch) || Self::is_opponent(state.player, ch) {
                    poss_moves.push((pos_x as u8, pos_y as u8));
                }
            }
        }

        poss_moves
    }

    fn mv_bishop(pos: Point<u8>, state: &BoardState) -> Vec<(u8, u8)> {
        let mut moves = Vec::<(u8, u8)>::with_capacity(13);
        moves.append(&mut Self::get_line_moves(&pos, Point::new( 1,  1), state));
        moves.append(&mut Self::get_line_moves(&pos, Point::new( 1, -1), state));
        moves.append(&mut Self::get_line_moves(&pos, Point::new(-1, -1), state));
        moves.append(&mut Self::get_line_moves(&pos, Point::new(-1,  1), state));
        moves
    }

    fn mv_rook(pos: Point<u8>, state: &BoardState) -> Vec<(u8, u8)> {
        let mut moves = Vec::<(u8, u8)>::with_capacity(14);
        for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
            moves.append(&mut Self::get_line_moves(&pos, Point::new(*dx, *dy), state));
        }

        moves
    }

    fn mv_queen(pos: Point<u8>, state: &BoardState) -> Vec<(u8, u8)> {
        let mut moves = Vec::<(u8, u8)>::with_capacity(28);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                moves.append(&mut Self::get_line_moves(&pos, Point::new(dx, dy), state));
            }
        }

        moves
    }

    fn mv_king(pos: Point<u8>, state: &BoardState) -> Vec<(u8, u8)> {
        let mut moves = Vec::<(u8, u8)>::with_capacity(8);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if pos.x == 0 && pos.y == 0 {
                    continue;
                }

                let x_m = pos.x as i8 + dx;
                let y_m = pos.y as i8 + dy;

                if x_m >= 0 && x_m < 8 && y_m >= 0 && y_m < 8 {
                    let ch = state.board[y_m as usize][x_m as usize];
                    if !Self::is_piece(ch) || Self::is_opponent(state.player, ch) {
                        moves.push((x_m as u8, y_m as u8));
                    }
                }
            }
        }

        let ch = state.board[pos.y as usize][pos.x as usize];

        let (checked, q_side, k_side, plyr) = match ch {
            'k' => (
                state.b_check,
                state.castling & 0b0010 > 0,
                state.castling & 0b0001 > 0,
                Player::Black,
            ),
            'K' => (
                state.w_check,
                state.castling & 0b1000 > 0,
                state.castling & 0b0100 > 0,
                Player::White,
            ),
            _ => (false, false, false, Player::White),
        };

        if checked {
            return moves;
        }

        let y = pos.y as usize;

        if k_side {
            let mut accept = true;
            for x in 5..=6 {
                if Self::is_piece(state.board[y][x as usize]) {
                    accept = false;
                    break;
                }
                let mut state_ = state.clone();
                Self::move_piece_to(pos.clone(), Point::new(x, pos.y), &mut state_);
                let checked = match plyr {
                    Player::White => state_.w_check,
                    Player::Black => state_.b_check,
                };

                if checked {
                    accept = false;
                    break;
                }
            }

            if accept {
                moves.push((6, pos.y));
            }
        }

        if q_side {
            let mut accept = true;
            for x in 2..=3 {
                if Self::is_piece(state.board[y][x as usize]) {
                    accept = false;
                    break;
                }
                let mut state_ = state.clone();
                Self::move_piece_to(pos.clone(), Point::new(x, pos.y), &mut state_);
                let checked = match plyr {
                    Player::White => state_.w_check,
                    Player::Black => state_.b_check,
                };

                if checked {
                    accept = false;
                    break;
                }
            }

            if accept {
                moves.push((2, pos.y));
            }
        }

        moves
    }

    fn get_piece_moves(ch: char, pos: Point<u8>, state: &BoardState) -> Vec<(u8, u8)> {
        match ch {
            'p' | 'P' => Self::mv_pawn(pos, state),
            'r' | 'R' => Self::mv_rook(pos, state),
            'n' | 'N' => Self::mv_knight(pos, state),
            'b' | 'B' => Self::mv_bishop(pos, state),
            'q' | 'Q' => Self::mv_queen(pos, state),
            'k' | 'K' => Self::mv_king(pos, state),
            _ => Vec::<(u8, u8)>::new(),
        }
    }

    fn select_piece(&mut self, x: u8, y: u8) {
        let ch = self.board_pcs[y as usize][x as usize];

        if !Self::is_piece(ch) || Self::is_opponent(self.turn, ch) {
            return;
        }

        let board_state = self.get_board_state();

        let moves = Self::get_piece_moves(ch, Point::new(x, y), &board_state);

        self.current = Some(ch);
        self.current_pos = Some((x, y));

        for (m_x, m_y) in &moves {
            let mut state = board_state.clone();
            Self::move_piece_to(Point::new(x, y), Point::new(*m_x, *m_y), &mut state);
            let checked = match self.turn {
                Player::White => state.w_check,
                Player::Black => state.b_check,
            };

            if checked {
                continue;
            }

            self.board[*m_y as usize][*m_x as usize] = Color::from_rgb(200, 200, 0);

            self.moves.push((*m_x, *m_y));
        }

        self.board[y as usize][x as usize] = Color::from_rgb(255, 85, 85);

        self.needs_draw = true;
        self.moving = true;
    }

    fn is_white_to_move(&mut self) -> bool{
        match self.turn {
            Player::White => true,
            Player::Black => false,
        }
    }

    fn move_piece(&mut self, x: u8, y: u8) -> bool {
        if self.moves.contains(&(x, y)) {
            let mut state = self.get_board_state();
            let curr = self.current_pos.unwrap();
            Self::move_piece_to(Point::new(curr.0, curr.1), Point::new(x, y), &mut state);
            self.set_state(&state);
            self.current = None;
            self.current_pos = None;
            self.moving = false;
            self.turn = self.turn.switch();
            state.player = state.player.switch();
            self.moves.clear();
            self.needs_draw = true;
            self.reset_board();

            if Self::check_for_checkmate(self.turn, &state) {
                return true;
            } else {
                return false;
            }
        }

        let ch = self.board_pcs[y as usize][x as usize];

        if Self::is_piece(ch) && !Self::is_opponent(self.turn, ch) {
            self.moves.clear();
            self.reset_board();
            self.select_piece(x, y);
            self.needs_draw = true;
        }

        false
    }

    fn set_state(&mut self, state: &BoardState) {
        self.board_pcs = state.board;
        self.w_king_pos = state.wk_pos;
        self.b_king_pos = state.bk_pos;
        self.enp_b = state.enp_b;
        self.enp_w = state.enp_w;
        self.castling = state.castling;
        self.w_check = state.w_check;
        self.b_check = state.b_check;
        self.w_checks = state.w_checks;
        self.b_checks = state.b_checks;
    }

    fn check_for_checks(plyr: Player, state: &mut BoardState) -> bool {
        let orig = state.player;
        state.player = plyr.switch();
        for y in 0..8 {
            for x in 0..8 {
                let ch = state.board[y as usize][x as usize];

                let is_valid_piece = match plyr {
                    Player::White => Self::is_black_piece(ch),
                    Player::Black => Self::is_white_piece(ch),
                };

                if !is_valid_piece {
                    continue;
                }

                let k_pos = match plyr {
                    Player::White => &state.wk_pos,
                    Player::Black => &state.bk_pos,
                };

                if Self::get_piece_moves(ch, Point::new(x, y), state).contains(k_pos) {
                    state.player = orig;
                    return true;
                }
            }
        }

        state.player = orig;
        false
    }

    fn check_for_checkmate(plyr: Player, state: &BoardState) -> bool {
        if  state.board[4][4] == 'k' || state.board[4][4] == 'K' ||
            state.board[3][4] == 'k' || state.board[3][4] == 'K' ||
            state.board[4][3] == 'k' || state.board[4][3] == 'K' ||
            state.board[3][3] == 'k' || state.board[3][3] == 'K' ||
            state.w_checks == 3 || state.b_checks == 3  {
                return true;
        } else {
            for y in 0..8 {
                for x in 0..8 {
                    let ch = state.board[y as usize][x as usize];

                    let is_valid_piece = match plyr {
                        Player::White => Self::is_white_piece(ch),
                        Player::Black => Self::is_black_piece(ch),
                    };

                    if !is_valid_piece {
                        continue;
                    }

                    for (m_x, m_y) in Self::get_piece_moves(ch, Point::new(x, y), state) {
                        let mut state_ = state.clone();
                        Self::move_piece_to(Point::new(x, y), Point::new(m_x, m_y), &mut state_);

                        let checked = match plyr {
                            Player::White => state_.w_check,
                            Player::Black => state_.b_check,
                        };

                        if !checked {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
}

impl EventHandler<ggez::GameError> for RChess {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.needs_draw {
            return Ok(());
        }
        graphics::clear(ctx, Color::from_rgb(0, 0, 0));

        for (y, row) in self.board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let x_sq = x as i32 * self.sq_size;
                let y_sq = y as i32 * self.sq_size;

                let r = graphics::Rect::new_i32(x_sq, y_sq, self.sq_size, self.sq_size);
                let mesh = graphics::Mesh::new_rectangle(ctx, DrawMode::fill(), r, *cell)?;

                graphics::draw(ctx, &mesh, DrawParam::default())?;

                let ch = self.board_pcs[y][x];

                if Self::is_piece(ch) {
                    let img = match self.pieces.get(&ch) {
                        Some(i) => i,
                        None => continue,
                    };

                    let ddraw = (self.sq_size as f32 - img.width() as f32 * 1.5) / 2.;
                    let x_draw = x_sq as f32 + ddraw;
                    let y_draw = y_sq as f32 + ddraw;
                    let draw_param = DrawParam::new().dest([x_draw, y_draw]).scale([1.5, 1.5]);

                    graphics::draw(ctx, img, draw_param)?;
                }
            }
        }
        let w_t = format!("{}{}", "White checks: ", self.w_checks);
        let w_msg= graphics::Text::new(w_t);
        let w_dest: ggez::mint::Point2<f32> = ggez::mint::Point2{x:830.0, y:500.0};
        graphics::draw(ctx, &w_msg, (w_dest, 0.0, Color::RED))?;

        let b_t = format!("{}{}", "Black checks: ", self.b_checks);
        let b_msg= graphics::Text::new(b_t);
        let b_dest: ggez::mint::Point2<f32> = ggez::mint::Point2{x:830.0, y:300.0};
        graphics::draw(ctx, &b_msg, (b_dest, 0.0, Color::RED))?;

        self.needs_draw = false;
        if self.w_win {
            graphics::clear(ctx, Color::from_rgb(0, 0, 0)); 
            let msg= graphics::Text::new("White won!");
            let dest: ggez::mint::Point2<f32> = ggez::mint::Point2{x:400.0, y:400.0};
            graphics::draw(ctx, &msg, (dest, 0.0, Color::RED))?;
        }
        if self.b_win {
            graphics::clear(ctx, Color::from_rgb(0, 0, 0));
            let msg= graphics::Text::new("Black won!");
            let dest: ggez::mint::Point2<f32> = ggez::mint::Point2{x:400.0, y:400.0};
            graphics::draw(ctx, &msg, (dest, 0.0, Color::RED))?;
        }
        graphics::present(ctx)
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, btn: MouseButton, x: f32, y: f32) {
        let x = (x as i32 / self.sq_size) as u8;
        let y = (y as i32 / self.sq_size) as u8;

        match btn {
            MouseButton::Left => {
                if !self.moving {
                    self.select_piece(x, y);
                } else {
                    let mated = self.move_piece(x, y);

                    if mated {
                        if  self.is_white_to_move(){
                            self.black_won();
                        } else {
                            self.white_won();
                        }
                    }
                }
            }

            _ => (),
        }
    }
}
