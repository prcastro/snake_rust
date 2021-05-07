use ggez::*;
use ggez::event::{KeyCode, KeyMods};
use oorandom::Rand32;
use getrandom;
use std::collections::LinkedList;
use std::time::{Duration, Instant};

const GRID_SIZE: (i32, i32) = (30, 20);
const GRID_CELL_SIZE: (i32, i32) = (32, 32);
const SCREEN_SIZE: (f32, f32) = (
    (GRID_SIZE.0 * GRID_CELL_SIZE.0) as f32,
    (GRID_SIZE.1 * GRID_CELL_SIZE.1) as f32,
);
const UPDATES_PER_SECOND: f32 = 8.0;
const TIME_PER_UPDATE: Duration = Duration::from_millis((1.0 / UPDATES_PER_SECOND * 1000.0) as u64);

#[derive(Clone, Copy, PartialEq)]
struct GridElem {
    x: i32,
    y: i32
}

impl From<GridElem> for graphics::Rect {
    fn from(pos: GridElem) -> Self {
        graphics::Rect::new_i32(
            pos.x * GRID_CELL_SIZE.0,
            pos.y * GRID_CELL_SIZE.1,
            GRID_CELL_SIZE.0,
            GRID_CELL_SIZE.0)
    }
}

impl GridElem {
    fn random(rng: &mut Rand32, max_x: i32, max_y: i32) -> Self {
        GridElem {
            x: rng.rand_range(0..(max_x as u32)) as i32,
            y: rng.rand_range(0..(max_y as u32)) as i32
        }
    }

    fn move_dir(&mut self, direction: Direction) -> Self {
        match direction {
            Direction::Up => GridElem {
                x: self.x,
                y: (self.y - 1).rem_euclid(GRID_SIZE.1)
            },
            Direction::Down => GridElem {
                x: self.x,
                y: (self.y + 1).rem_euclid(GRID_SIZE.1)
            },
            Direction::Left => GridElem {
                x: (self.x - 1).rem_euclid(GRID_SIZE.0),
                y: self.y
            },
            Direction::Right => GridElem {
                x: (self.x + 1).rem_euclid(GRID_SIZE.0),
                y: self.y
            }
        }
    }

    fn draw(&self, ctx: &mut Context, color: graphics::Color) -> GameResult {
        let rectangle = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            (*self).into(),
            color
        )?;
        
        graphics::draw(ctx, &rectangle, graphics::DrawParam::default())
    }
}

fn same_position(elem1: &GridElem, elem2: &GridElem) -> bool {
    return *elem1 == *elem2
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    fn from_keycode(keycode: KeyCode) -> Option<Direction> {
        match keycode {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            _ => None
        }
    }

    fn inverse(&self) -> Self {
        match *self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

struct Snake {
    head: GridElem,
    body: LinkedList<GridElem>,
    ate: bool,
    self_ate: bool,
    direction: Direction
}

impl Snake {
    fn new(pos: GridElem) -> Self {
        let body_pos = GridElem {
            x: pos.x - 1,
            y: pos.y 
        };

        let mut body = LinkedList::new();
        body.push_back(body_pos);

        Snake {
            head: pos,
            body: body,
            ate: false,
            self_ate: false,
            direction: Direction::Right
        }
    }

    fn update(&mut self, food: &Food) -> GameResult<()> {
        let new_head = self.head.move_dir(self.direction);

        self.ate = same_position(&new_head, &(food.elem));
        self.body.push_front(self.head);

        // check if self ate
        let mut self_ate = false;
        for body_elem in self.body.iter() {
            if same_position(&new_head, body_elem) {
                self_ate = true;
            }
        }
        self.self_ate = self_ate;

        if !self.ate {
            self.body.pop_back();
        }

        self.head = new_head;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()>  {
        for elem in self.body.iter() {
            elem.draw(ctx, graphics::WHITE)?;
        }
        self.head.draw(ctx, graphics::WHITE)
    }
}

struct Food {
    elem: GridElem
}

impl Food {
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.elem.draw(ctx, [0.0, 255.0, 0.0, 1.0].into())
    }
}

struct State {
    snake: Snake,
    food: Food,
    rng: Rand32,
    last_update_time: Instant
}

impl State {
    fn new() -> State {
        // And we seed our RNG with the system RNG.
        let mut seed: [u8; 8] = [0; 8];
        getrandom::getrandom(&mut seed).expect("Could not create RNG seed");

        State {
            snake: Snake::new(GridElem { x: 15, y: 10 }),
            food: Food {
                elem: GridElem { x: 5, y: 5 }
            },
            rng: Rand32::new(u64::from_ne_bytes(seed)),
            last_update_time: Instant::now()
        }
    }
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let time_since_last_update = Instant::now() - self.last_update_time;
        if time_since_last_update < TIME_PER_UPDATE {
            return Ok(())
        };

        self.snake.update(&self.food)?;

        if self.snake.ate {
            self.food.elem = GridElem::random(&mut self.rng, GRID_SIZE.0, GRID_SIZE.1);
        }

        if self.snake.self_ate {
            println!("GAME OVER!");
            event::quit(ctx);
        }

        self.last_update_time = Instant::now();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);
        self.food.draw(ctx)?;
        self.snake.draw(ctx)?;
        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        if let Some(direction) = Direction::from_keycode(keycode) {
            if direction.inverse() != self.snake.direction {
                self.snake.direction = direction;
            }
        }
    }
}

fn main() {
    let mut state = State::new();

    let (ref mut ctx, ref mut even_loop) = ContextBuilder::new("snake", "prcastro")
        .window_setup(conf::WindowSetup::default().title("Snake").vsync(true))
        .window_mode(conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()
        .unwrap();
    
    event::run(ctx, even_loop, &mut state).unwrap();
}
