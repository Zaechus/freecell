use std::io::{stdout, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
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

    let freecells = ["(A)", "(B)", "(C)", "(D)"];
    let ace_piles = ["(E)", "(F)", "(G)", "(H)"];

    let mut cascades: [Vec<&str>; 8] = [
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];

    for (i, card) in deck.drain(..).enumerate() {
        cascades[i % 8].push(card);
    }

    let mut cursor_pos: (u16, u16) = (0, 0);

    enable_raw_mode().unwrap();
    let mut stdout = stdout();
    execute!(stdout, terminal::Clear(terminal::ClearType::All),).unwrap();

    'outer: loop {
        execute!(stdout, terminal::Clear(terminal::ClearType::All),).unwrap();
        execute!(stdout, cursor::MoveTo(0, 0)).unwrap();

        print!("\t");
        for x in freecells {
            print!("{}\t", x)
        }
        for x in ace_piles {
            print!("{}\t", x)
        }
        write!(stdout, "\r\n\r\n").unwrap();

        let longest_len = cascades.iter().max_by_key(|col| col.len()).unwrap().len();
        for i in 0..longest_len {
            print!("\t");
            for col in 0..8 {
                let s = cascades[col].get(i).unwrap_or(&"   ");
                if stdout.is_tty() {
                    print!("{}\t", s.with(card_color(s)));
                } else {
                    print!("{}\t", s)
                }
            }
            write!(stdout, "\r\n").unwrap();
        }

        let mut vertical_max = (cascades[cursor_pos.0 as usize].len() + 1) as u16;
        loop {
            execute!(stdout, cursor::MoveTo(8 + cursor_pos.0 * 8, cursor_pos.1)).unwrap();

            let event = event::read().unwrap();

            if event == Event::Key(KeyCode::Char('q').into()) {
                quit()
            } else if event == Event::Key(KeyCode::Up.into())
                || event == Event::Key(KeyCode::Char('k').into())
            {
                cursor_pos.1 = cursor_pos.1.saturating_sub(1);
                if cursor_pos.1 == 1 {
                    cursor_pos.1 = 0;
                }
            } else if event == Event::Key(KeyCode::Down.into())
                || event == Event::Key(KeyCode::Char('j').into())
            {
                cursor_pos.1 = cursor_pos.1.saturating_add(1);
                if cursor_pos.1 == 1 {
                    cursor_pos.1 = 2;
                }
            } else if event == Event::Key(KeyCode::Left.into())
                || event == Event::Key(KeyCode::Char('h').into())
            {
                cursor_pos.0 = cursor_pos.0.saturating_sub(1).clamp(0, 7);
                vertical_max = (cascades[cursor_pos.0 as usize].len() + 1) as u16;
            } else if event == Event::Key(KeyCode::Right.into())
                || event == Event::Key(KeyCode::Char('l').into())
            {
                cursor_pos.0 = cursor_pos.0.saturating_add(1).clamp(0, 7);
                vertical_max = (cascades[cursor_pos.0 as usize].len() + 1) as u16;
            } else if event == Event::Key(KeyCode::Esc.into()) {
                continue 'outer;
            } else if event == Event::Key(KeyCode::Enter.into()) {
                break;
            }

            cursor_pos.1 = cursor_pos.1.clamp(0, vertical_max);
        }
        let pick_pos = (cursor_pos.0 as usize, (cursor_pos.1 - 2) as usize);

        execute!(
            stdout,
            cursor::MoveTo((cursor_pos.0 + 1) * 8 - 1, cursor_pos.1),
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
            } else if event == Event::Key(KeyCode::Enter.into()) {
                break;
            }
        }
        let place_column = cursor_pos.0 as usize;

        if can_place(
            cascades[pick_pos.0][pick_pos.1],
            cascades[place_column].last().unwrap(),
        ) {
            let card = cascades[pick_pos.0].remove(pick_pos.1);
            cascades[place_column].push(card);
        }
    }
}

fn can_place(picked: &str, place: &str) -> bool {
    card_value(picked) == (card_value(place) - 1) && card_color(picked) != card_color(place)
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
