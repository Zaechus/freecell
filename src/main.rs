use std::io::{stdout, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Color, Print, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    tty::IsTty,
};
use rand::{seq::SliceRandom, thread_rng};

fn main() {
    let mut deck = vec![
        "A ♠", "2 ♠", "3 ♠", "4 ♠", "5 ♠", "6 ♠", "7 ♠", "8 ♠", "9 ♠", "10♠", "J ♠", "Q ♠", "K ♠",
        "A ❤", "2 ❤", "3 ❤", "4 ❤", "5 ❤", "6 ❤", "7 ❤", "8 ❤", "9 ❤", "10❤", "J ❤", "Q ❤", "K ❤",
        "A ♣", "2 ♣", "3 ♣", "4 ♣", "5 ♣", "6 ♣", "7 ♣", "8 ♣", "9 ♣", "10♣", "J ♣", "Q ♣", "K ♣",
        "A ♦", "2 ♦", "3 ♦", "4 ♦", "5 ♦", "6 ♦", "7 ♦", "8 ♦", "9 ♦", "10♦", "J ♦", "Q ♦", "K ♦",
    ];
    deck.shuffle(&mut thread_rng());

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
    let mut stdout = stdout();
    queue!(stdout, terminal::Clear(terminal::ClearType::All),).unwrap();

    'outer: loop {
        queue!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )
        .unwrap();

        print!("\t");
        for col in &cards {
            if stdout.is_tty() {
                print!("{}\t", col[0].with(card_color(col[0])))
            } else {
                print!("{}\t", col[0])
            }
        }
        print!("\r\n\r\n");

        let longest_len = cards.iter().max_by_key(|col| col.len()).unwrap().len();
        for i in 1..longest_len {
            print!("\t");
            for cascade in &cards {
                let s = cascade.get(i).unwrap_or(&"   ");
                if stdout.is_tty() {
                    print!("{}\t", s.with(card_color(s)));
                } else {
                    print!("{}\t", s)
                }
            }
            print!("\r\n");
        }
        stdout.flush().unwrap();

        let mut vertical_max = (cards[cursor_pos.0 as usize].len() + 1) as u16;
        loop {
            execute!(
                stdout,
                cursor::MoveTo(
                    8 + cursor_pos.0 * 8,
                    if cursor_pos.1 == 0 {
                        0
                    } else {
                        cursor_pos.1 + 1
                    }
                )
            )
            .unwrap();

            if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Char('h') | KeyCode::Left => {
                        cursor_pos.0 = cursor_pos.0.saturating_sub(1).clamp(0, 7);
                        vertical_max = (cards[cursor_pos.0 as usize].len() + 1) as u16;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        cursor_pos.1 = cursor_pos.1.saturating_add(1);
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        cursor_pos.1 = cursor_pos.1.saturating_sub(1);
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        cursor_pos.0 = cursor_pos.0.saturating_add(1).clamp(0, 7);
                        vertical_max = (cards[cursor_pos.0 as usize].len() + 1) as u16;
                    }
                    KeyCode::Char('q') => quit(),
                    KeyCode::Esc => continue 'outer,
                    KeyCode::Char(' ') | KeyCode::Enter => break,
                    _ => (),
                }
            }

            cursor_pos.1 = cursor_pos.1.clamp(0, vertical_max - 2);
        }
        if cursor_pos.1 == 0 && cursor_pos.0 > 3 {
            continue;
        }
        let pick_pos = (cursor_pos.0 as usize, cursor_pos.1 as usize);

        execute!(
            stdout,
            cursor::MoveTo(
                (cursor_pos.0 + 1) * 8 - 1,
                if cursor_pos.1 == 0 {
                    0
                } else {
                    cursor_pos.1 + 1
                }
            ),
            Print("["),
            cursor::MoveToColumn((cursor_pos.0 + 1) * 8 + 4),
            Print("]"),
        )
        .unwrap();

        let mut top = false;
        loop {
            execute!(
                stdout,
                cursor::MoveTo(
                    8 + cursor_pos.0 * 8,
                    if top { 0 } else { (longest_len + 2) as u16 }
                )
            )
            .unwrap();

            let event = event::read().unwrap();

            if event == Event::Key(KeyCode::Char('q').into()) {
                quit()
            } else if event == Event::Key(KeyCode::Up.into())
                || event == Event::Key(KeyCode::Char('k').into())
                || event == Event::Key(KeyCode::Down.into())
                || event == Event::Key(KeyCode::Char('j').into())
            {
                top ^= true;
            } else if event == Event::Key(KeyCode::Left.into())
                || event == Event::Key(KeyCode::Char('h').into())
            {
                cursor_pos.0 = cursor_pos.0.saturating_sub(1).clamp(0, 7);
            } else if event == Event::Key(KeyCode::Right.into())
                || event == Event::Key(KeyCode::Char('l').into())
            {
                cursor_pos.0 = cursor_pos.0.saturating_add(1).clamp(0, 7);
            } else if event == Event::Key(KeyCode::Esc.into()) {
                continue 'outer;
            } else if event == Event::Key(KeyCode::Enter.into())
                || event == Event::Key(KeyCode::Char(' ').into())
            {
                break;
            }
        }
        let place_column = cursor_pos.0 as usize;
    }
}

fn can_place(picked: &str, place: &str) -> bool {
    (card_value(picked) + 1) == card_value(place) && card_color(picked) != card_color(place)
        || place == "   "
}

fn can_move_to_foundation(card: &str, foundation_top: &str) -> bool {
    (card_value(card) - 1) == card_value(foundation_top)
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
        '❤' | '♦' => Color::Red,
        _ => Color::Reset,
    }
}

fn quit() {
    disable_raw_mode().unwrap();
    std::process::exit(0);
}
