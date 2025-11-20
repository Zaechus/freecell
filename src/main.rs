use std::{cmp::Ordering, env, io::stdout, process::ExitCode};

use crossterm::{
    cursor::{MoveTo, MoveToColumn, SetCursorStyle},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute, queue,
    style::{Color, Print, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
    tty::IsTty,
};
use rand::seq::SliceRandom;

fn main() -> ExitCode {
    let mut stdout = stdout();
    if !stdout.is_tty() {
        return ExitCode::FAILURE;
    }

    let restrict_movement = !env::args().any(|arg| arg == "--no-restrict");

    let mut deck = vec![
        "A ♠", "2 ♠", "3 ♠", "4 ♠", "5 ♠", "6 ♠", "7 ♠", "8 ♠", "9 ♠", "10♠", "J ♠", "Q ♠", "K ♠",
        "A ♥", "2 ♥", "3 ♥", "4 ♥", "5 ♥", "6 ♥", "7 ♥", "8 ♥", "9 ♥", "10♥", "J ♥", "Q ♥", "K ♥",
        "A ♣", "2 ♣", "3 ♣", "4 ♣", "5 ♣", "6 ♣", "7 ♣", "8 ♣", "9 ♣", "10♣", "J ♣", "Q ♣", "K ♣",
        "A ♦", "2 ♦", "3 ♦", "4 ♦", "5 ♦", "6 ♦", "7 ♦", "8 ♦", "9 ♦", "10♦", "J ♦", "Q ♦", "K ♦",
    ];
    deck.shuffle(&mut rand::rng());

    let mut cards: [Vec<&str>; 8] = [
        vec!["   "],
        vec!["   "],
        vec!["   "],
        vec!["   "],
        vec!["   "],
        vec!["   "],
        vec!["   "],
        vec!["   "],
    ];

    for (i, card) in deck.drain(..).enumerate() {
        cards[i % 8].push(card);
    }

    let mut cursor_pos: (u16, u16) = (0, 0);

    enable_raw_mode().unwrap();
    queue!(stdout, SetCursorStyle::SteadyBlock, Clear(ClearType::All)).unwrap();

    'outer: loop {
        queue!(stdout, MoveTo(0, 0), Clear(ClearType::FromCursorDown),).unwrap();

        let terminal_width = terminal::size().unwrap().0;
        let tab: u16 = if terminal_width > 72 {
            8
        } else if terminal_width > 54 {
            6
        } else {
            4
        };
        let utab: usize = tab.into();

        print!("{:utab$}", ' ');
        for col in &cards {
            print!("{}", format!("{:utab$}", col[0]).with(card_color(col[0])));
        }
        print!("\r\n\r\n");

        let longest_len = cards.iter().max_by_key(|col| col.len()).unwrap().len();
        for i in 1..longest_len {
            print!("{:utab$}", ' ');
            for cascade in &cards {
                let s = cascade.get(i).unwrap_or(&"   ");
                print!("{}", format!("{s:utab$}").with(card_color(s)));
            }
            print!("\r\n");
        }

        loop {
            cursor_pos.1 = cursor_pos
                .1
                .clamp(0, (cards[cursor_pos.0 as usize].len() + 1) as u16 - 2);

            execute!(
                stdout,
                MoveTo(
                    tab + cursor_pos.0 * tab,
                    if cursor_pos.1 == 0 {
                        0
                    } else {
                        cursor_pos.1 + 1
                    }
                )
            )
            .unwrap();

            if let Event::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            }) = event::read().unwrap()
            {
                match code {
                    KeyCode::Char('h') | KeyCode::Left => {
                        cursor_pos.0 = cursor_pos.0.saturating_sub(1).clamp(0, 7);
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        cursor_pos.1 = cursor_pos.1.saturating_add(1);
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        cursor_pos.1 = cursor_pos.1.saturating_sub(1);
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        cursor_pos.0 = cursor_pos.0.saturating_add(1).clamp(0, 7);
                    }
                    KeyCode::Char('K') | KeyCode::PageUp => cursor_pos.1 = 0,
                    KeyCode::Char('J') | KeyCode::PageDown => {
                        cursor_pos.1 = cards[cursor_pos.0 as usize].len() as u16
                    }
                    KeyCode::Char('q') => return quit(),
                    KeyCode::Esc => continue 'outer,
                    KeyCode::Char(' ') | KeyCode::Enter => break,
                    _ => (),
                }
            }
        }
        let pick_pos = (cursor_pos.0 as usize, cursor_pos.1 as usize);
        if pick_pos.0 > 3 && pick_pos.1 == 0 || cards[pick_pos.0][pick_pos.1] == "   " {
            continue;
        }

        let selected_cards = if pick_pos.1 > 0 {
            &cards[pick_pos.0][pick_pos.1..]
        } else {
            &cards[pick_pos.0][pick_pos.1..1]
        };

        if selected_cards.len() > 1 {
            for i in 1..selected_cards.len() {
                if !can_move(selected_cards[i], selected_cards[i - 1]) {
                    continue 'outer;
                }
            }
        }

        execute!(
            stdout,
            MoveTo(
                (cursor_pos.0 + 1) * tab - 1,
                if cursor_pos.1 == 0 {
                    0
                } else {
                    cursor_pos.1 + 1
                }
            ),
            Print("["),
            MoveToColumn((cursor_pos.0 + 1) * tab + if tab > 4 { 4 } else { 3 }),
            Print("]"),
        )
        .unwrap();

        let mut moving_to_top_row = false;
        loop {
            cursor_pos.1 = if moving_to_top_row {
                0
            } else {
                (longest_len + 1) as u16
            };
            execute!(stdout, MoveTo(tab + cursor_pos.0 * tab, cursor_pos.1)).unwrap();

            if let Event::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            }) = event::read().unwrap()
            {
                match code {
                    KeyCode::Char('h') | KeyCode::Left => {
                        cursor_pos.0 = cursor_pos.0.saturating_sub(1).clamp(0, 7);
                    }
                    KeyCode::Char('j') | KeyCode::Down | KeyCode::Char('k') | KeyCode::Up => {
                        moving_to_top_row ^= true;
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        cursor_pos.0 = cursor_pos.0.saturating_add(1).clamp(0, 7);
                    }
                    KeyCode::Char('q') => return quit(),
                    KeyCode::Esc => continue 'outer,
                    KeyCode::Char(' ') | KeyCode::Enter => break,
                    _ => (),
                }
            }
        }
        let place_column = cursor_pos.0 as usize;

        if restrict_movement {
            let free_cells = cards[..4].iter().filter(|col| col[0] == "   ").count();
            let max_selected = 1
                + free_cells
                + cards.iter().filter(|col| col.len() == 1).count() * (free_cells + 1)
                - (cards[place_column].len() == 1) as usize * (free_cells + 1);
            if selected_cards.len() > max_selected {
                continue;
            }
        }

        if moving_to_top_row
            && selected_cards.len() == 1
            && (place_column < 4 && cards[place_column][0] == "   "
                || can_move_to_foundation(selected_cards[0], cards[place_column][0]))
        {
            if pick_pos.1 == 0 {
                let tmp = cards[pick_pos.0][pick_pos.1];
                cards[pick_pos.0][pick_pos.1] = "   ";
                cards[place_column][0] = tmp;
            } else {
                let tmp = cards[pick_pos.0].remove(pick_pos.1);
                cards[place_column][0] = tmp;
            }
        } else if cards[place_column].len() == 1
            || can_move(
                selected_cards[0],
                cards[place_column].last().unwrap_or(&"   "),
            )
        {
            match selected_cards.len().cmp(&1) {
                Ordering::Equal => {
                    if pick_pos.1 == 0 {
                        cards[place_column].push(cards[pick_pos.0][pick_pos.1]);
                        cards[pick_pos.0][pick_pos.1] = "   ";
                    } else {
                        let tmp = cards[pick_pos.0].remove(pick_pos.1);
                        cards[place_column].push(tmp);
                    }
                }
                Ordering::Greater => {
                    let tmp: Vec<_> = cards[pick_pos.0].drain(pick_pos.1..).collect();
                    cards[place_column].extend(tmp);
                }
                Ordering::Less => (),
            }
        }
    }
}

fn can_move(picked: &str, place: &str) -> bool {
    card_value(picked) + 1 == card_value(place) && card_color(picked) != card_color(place)
}

fn can_move_to_foundation(card: &str, foundation_top: &str) -> bool {
    card_value(card) - 1 == card_value(foundation_top)
        && card.chars().nth(2) == foundation_top.chars().nth(2)
        || card_value(card) == 1 && card_value(foundation_top) == 0
}

fn card_value(s: &str) -> u8 {
    match &s[0..2] {
        "A " => 1,
        "2 " => 2,
        "3 " => 3,
        "4 " => 4,
        "5 " => 5,
        "6 " => 6,
        "7 " => 7,
        "8 " => 8,
        "9 " => 9,
        "10" => 10,
        "J " => 11,
        "Q " => 12,
        "K " => 13,
        _ => 0,
    }
}

fn card_color(s: &str) -> Color {
    match s.chars().nth(2).unwrap() {
        '♥' | '♦' => Color::Red,
        _ => Color::Reset,
    }
}

fn quit() -> ExitCode {
    execute!(
        stdout(),
        SetCursorStyle::DefaultUserShape,
        MoveTo(0, 0),
        Clear(ClearType::All)
    )
    .unwrap();
    disable_raw_mode().unwrap();
    ExitCode::SUCCESS
}
