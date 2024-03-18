use console::{Key, Term};
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use std::cmp::{max, min};

use Pos::*;
use Split::*;

const ROOM_LEVEL: u16 = 4;

#[derive(Clone)]
enum Pos {
    First,
    Last,
}

#[derive(Clone)]
enum Split {
    Ver(Option<Pos>),
    Hor(Option<Pos>),
}

fn rnd_split(size: u16) -> u16 {
    let mut rng = thread_rng();

    let w = size as f64 * 0.15;
    let wiggle = rng.gen_range(-w..=w);

    ((size as f64 / 2.0) + wiggle).floor() as u16
}

fn print_corridor(term: &Term, split: Split, mid: u16, from: (u16, u16), to: (u16, u16)) {
    let (fx, fy) = from;
    let (tx, ty) = to;

    let mut coords: Vec<(usize, usize)> = vec![];

    // TODO: DRY
    match split {
        Ver(_) => {
            for x in fx..=mid {
                coords.push((x as usize, fy as usize));
            }

            for x in mid..=tx {
                coords.push((x as usize, ty as usize));
            }

            let s = min(fy, ty);
            let t = max(fy, ty);

            for y in s..=t {
                coords.push((mid as usize, y as usize));
            }
        }
        Hor(_) => {
            for y in fy..=mid {
                coords.push((fx as usize, y as usize));
            }

            for y in mid..=ty {
                coords.push((tx as usize, y as usize));
            }

            let s = min(fx, tx);
            let t = max(fx, tx);

            for x in s..=t {
                coords.push((x as usize, mid as usize));
            }
        }
    }

    for (x, y) in coords {
        term.move_cursor_to(x, y).unwrap();
        term.write_str("░").unwrap();
    }
}

fn print_division(
    term: &Term,
    split: Split,
    left: u16,
    top: u16,
    cols: u16,
    rows: u16,
    level: u16,
) -> (u16, u16) {
    if level == ROOM_LEVEL {
        let mut rng = thread_rng();

        let min_cols: u16 = max(1u16, ((cols as f64) * 0.7).floor() as u16);
        let max_cols: u16 = cols;

        let min_rows: u16 = max(1u16, ((rows as f64) * 0.7).floor() as u16);
        let max_rows: u16 = rows;

        let new_cols = rng.gen_range(min_cols..=max_cols);
        let new_rows = rng.gen_range(min_rows..=max_rows);

        let left = rng.gen_range(left..=(left + (cols - new_cols)));
        let cols = rng.gen_range(top..=(top + (rows - new_rows)));

        let cols = new_cols;
        let rows = new_rows;

        for x in left..=(left + cols) {
            term.move_cursor_to(x as usize, top as usize).unwrap();
            term.write_str("-").unwrap();
            term.move_cursor_to(x as usize, (top + rows) as usize)
                .unwrap();
            term.write_str("-").unwrap();
        }

        for y in (top + 1)..(top + rows) {
            term.move_cursor_to(left as usize, y as usize).unwrap();
            term.write_str("|").unwrap();
            term.move_cursor_to((left + cols) as usize, y as usize)
                .unwrap();
            term.write_str("|").unwrap();
        }

        return match split {
            Ver(Some(pos)) => {
                let door_x = rng.gen_range((left + 1)..(left + cols));

                match pos {
                    First => (door_x, top + rows),
                    Last => (door_x, top),
                }
            }
            Hor(Some(pos)) => {
                let door_y = rng.gen_range((top + 1)..(top + rows));

                match pos {
                    First => (left + cols, door_y),
                    Last => (left, door_y),
                }
            }
            _ => unreachable!("This can't happen."),
        };
    }

    let next_level = level + 1;

    match split {
        Ver(_) => {
            let sp = rnd_split(cols);

            let sub_left = print_division(
                term,
                Split::Hor(Some(First)),
                left,
                top,
                sp - 2,
                rows,
                next_level,
            );
            let sub_right = print_division(
                term,
                Split::Hor(Some(Last)),
                left + sp + 2,
                top,
                cols - sp - 2,
                rows,
                next_level,
            );

            let min_y = min(sub_left.1, sub_right.1);
            let max_y = max(sub_left.1, sub_right.1);

            let entrance_x = left + sp;
            let entrance_y = min_y + (max_y - min_y) / 2;

            print_corridor(term, split, entrance_x, sub_left, sub_right);

            (entrance_x, entrance_y)
        }
        Hor(_) => {
            let sp = rnd_split(rows);

            let sub_top = print_division(
                term,
                Split::Ver(Some(First)),
                left,
                top,
                cols,
                sp - 2,
                next_level,
            );
            let sub_bot = print_division(
                term,
                Split::Ver(Some(Last)),
                left,
                top + sp + 2,
                cols,
                rows - sp - 2,
                next_level,
            );

            let min_x = min(sub_top.0, sub_bot.0);
            let max_x = max(sub_top.0, sub_bot.0);

            let entrance_x = min_x + (max_x - min_x) / 2;
            let entrance_y = top + sp;

            print_corridor(term, split, entrance_y, sub_top, sub_bot);

            (entrance_x, entrance_y)
        }
    }
}

fn print_dungeon(term: &Term) {
    let mut rng = thread_rng();

    let (rows, cols) = term.size();

    let mut vars = [Ver(None), Hor(None)];
    vars.shuffle(&mut rng);

    print_division(&term, vars[0].clone(), 0, 0, cols, rows, 0);

    term.move_cursor_to(cols as usize, rows as usize).unwrap();
}

fn main() {
    let term = Term::buffered_stdout();

    loop {
        term.clear_screen().unwrap();

        print_dungeon(&term);

        term.flush().unwrap();

        if let Ok(key) = term.read_key() {
            match key {
                Key::Escape => {
                    break;
                }
                _ => {}
            }
        } else {
            break;
        }
    }
}
