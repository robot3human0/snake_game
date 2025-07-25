extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use rand::prelude::*;

use std::collections::LinkedList;
use std::iter::FromIterator;

// Глобальные константы
const CELL_SIZE: i32 = 20;                                      // размер ячейки 20 пикселей
const GRID_SIZE: i32 = 20;                                      // Размер гряды/столбца 20 ячеек (400 пикселей)
const BOARD_SIZE: u32 = (GRID_SIZE * GRID_SIZE) as u32;         // Размер игрового поля 400 (х 400) пикселей
const INITIAL_SNAKE_BODY: &[(i32, i32)] = &[(1, 0), (0, 0)];    // Место на доске где появляется новая змейка (две ячейки по горизонтали в левом верхнем углу)
const FRAMES_PER_SECOND: u64 = 6;                               // Частота перерисовки кадров в секунду (скорость змейки)
#[derive(Clone, PartialEq, Debug)]
enum Direction {
    Right, Left, Up, Down
}
pub struct Game {
    gl: GlGraphics,
    snake: Snake,
    food: Food,
}

impl Game {
    fn render(&mut self, args: &RenderArgs) {
        use graphics;

        const LAVENDER: [f32; 4] = [0.7, 0.75, 1.0, 1.0];   // Здесь можно поиграться с цветом игрового поля

        self.gl.draw(args.viewport(), |_c, gl| {
            graphics::clear(LAVENDER, gl);
        });

        self.snake.render(&mut self.gl, args);

        self.food.render(&mut self.gl, args);
    }

    fn update(&mut self) {
        if let Some(&(head_x, head_y)) = self.snake.body.front() {
            if head_x == self.food.x && head_y == self.food.y {
                self.snake.grow();
                self.food = Food::new_random(&self.snake.body);
            }
        }

        if !self.snake.update() {
            self.snake = Snake::new();
            self.food = Food::new_random(&self.snake.body);
        }
    }

    fn pressed(&mut self, button: &Button) {
        let last_direction = self.snake.direction.clone();

        self.snake.direction = match button {
            &Button::Keyboard(Key::Up)
                if last_direction != Direction::Down => Direction::Up,
            &Button::Keyboard(Key::Down)
                if last_direction != Direction::Up => Direction::Down,
            &Button::Keyboard(Key::Left)
                if last_direction != Direction::Right => Direction::Left,
            &Button::Keyboard(Key::Right)
                if last_direction != Direction::Left => Direction::Right,
            _ => last_direction,
        };
    }
}

struct Snake {
    body: LinkedList<(i32, i32)>,
    direction: Direction,
    growing: bool,
}

impl Snake {
    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics;

        const GOLD_CRAYOLA: [f32; 4] = [1.0, 0.8, 0.47, 1.0];   // Здесь можно поиграться с цветом змейки

        let squares: Vec<graphics::types::Rectangle> = self.body
            .iter()
            .map(|&(x, y)| {
                graphics::rectangle::square(
                    (x * CELL_SIZE) as f64,
                    (y * CELL_SIZE) as f64,
                    CELL_SIZE as f64)
            })
            .collect();

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            squares.into_iter()
                .for_each(|square| graphics::rectangle(GOLD_CRAYOLA, square, transform, gl));
        });
    }

    fn update(&mut self) -> bool {
        let mut new_head = (*self.body.front().expect("Snake has no body")).clone();

        match self.direction {
            Direction::Right => new_head.0 += 1,
            Direction::Left => new_head.0 -= 1,
            Direction::Up => new_head.1 -= 1,
            Direction::Down => new_head.1 += 1,
        }

        if self.body.contains(&new_head) {
            return false;
        }

        // wrap around
        if new_head.0 < 0 {
            new_head.0 = GRID_SIZE - 1;
        } else if new_head.0 >= GRID_SIZE {
            new_head.0 = 0;
        }

        if new_head.1 < 0 {
            new_head.1 = GRID_SIZE - 1;
        } else if new_head.1 >= GRID_SIZE {
            new_head.1 = 0;
        }

        self.body.push_front(new_head);
        if self.growing {
            self.growing = false;
        } else {
            self.body.pop_back();
        }
        true
    }

    fn grow(&mut self) {
        self.growing = true;
    }

    fn new() -> Snake {
        Snake {
            body: LinkedList::from_iter(INITIAL_SNAKE_BODY.iter().cloned()),
            direction: Direction::Right,
            growing: false,
        }
    }
}

pub struct Food {
    x: i32,
    y: i32,
}

impl Food {
    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics;

        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0]; // Здесь можно поиграть с цветом еды

        let square = graphics::rectangle::square(
            (self.x * CELL_SIZE) as f64,
            (self.y * CELL_SIZE) as f64,
            CELL_SIZE as f64);

        gl.draw(args.viewport(), |c, gl| {
            graphics::rectangle(RED, square, c.transform, gl);
        });
    }

    fn new_random(snake_body: &LinkedList<(i32, i32)>) -> Food {
        let mut rng = rand::rng();

        let mut x = rng.random_range(0..GRID_SIZE);
        let mut y = rng.random_range(0..GRID_SIZE);

        // Проверка на что еда не появится в том месте, где расположена змейка.
        while snake_body.contains(&(x, y)) {
            x = rng.random_range(0..GRID_SIZE);
            y = rng.random_range(0..GRID_SIZE);
        }

        Food { x, y }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: GlutinWindow = WindowSettings::new(
        "Snake Game",
        [BOARD_SIZE, BOARD_SIZE]
    ).opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let gl = GlGraphics::new(opengl);
    let snake = Snake::new();
    let food = Food::new_random(&snake.body);
    let mut game = Game {gl, snake, food };

    let mut events = Events::new(EventSettings::new()).ups(FRAMES_PER_SECOND);
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(_u) = e.update_args() {
            game.update();
        }

        if let Some(k) = e.button_args() {
            if k.state == ButtonState::Press {
                game.pressed(&k.button);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_move_right() {
        let mut snake = Snake::new();

        let prev_head = snake.body.front().unwrap().clone();

        snake.direction = Direction::Right;
        snake.update();

        let current_head = snake.body.front().unwrap().clone();

        assert_ne!(prev_head, current_head);
        assert_eq!(snake.direction, Direction::Right);
    }

    #[test]
    fn snake_move_down() {
        let mut snake = Snake::new();

        let prev_head = snake.body.front().unwrap().clone();

        snake.direction = Direction::Down;
        snake.update();

        let current_head = snake.body.front().unwrap().clone();

        assert_ne!(prev_head, current_head);
        assert_eq!(snake.direction, Direction::Down);
    }

    #[test]
    fn snake_move_left() {
        // Для этого теста нужно создать кастомную змейку, потому что оригинальная движется в право
        // при рождении, соответственно мы не можем повернуть сразу влево.
        // Поэтому мы в этом тесте создаем змейку, которая движется вниз изначально!
        let mut snake = Snake {
            body: LinkedList::from_iter([(0, 1), (0, 0)]),
            direction: Direction::Down,
            growing: false,
        };

        let prev_head = snake.body.front().unwrap().clone();

        snake.direction = Direction::Left;
        snake.update();

        let current_head = snake.body.front().unwrap().clone();

        assert_ne!(prev_head, current_head);
        assert_eq!(snake.direction, Direction::Left);
    }

    #[test]
    fn snake_move_up() {
        let mut snake = Snake::new();

        let prev_head = snake.body.front().unwrap().clone();

        snake.direction = Direction::Up;
        snake.update();

        let current_head = snake.body.front().unwrap().clone();

        assert_ne!(prev_head, current_head);
        assert_eq!(snake.direction, Direction::Up);
    }

    #[test]
    fn snake_growth() {
        let mut snake = Snake::new();

        let initial_length = snake.body.len();

        snake.grow();
        snake.update();

        assert_ne!(initial_length, snake.body.len());
    }

    #[test]
    fn self_collision() {
        let mut snake = Snake {
            growing: false,
            direction: Direction::Up,
            body: LinkedList::from_iter([(0, 1), (1, 1), (1, 0), (0, 0)])
        };

        let result = snake.update();

        assert!(!result, "Snake should collide with itself");
    }

    #[test]
    fn snake_wraps_around() {
        let mut snake = Snake::new();

        snake.direction = Direction::Up;
        snake.update();

        let head_should_be = (1, GRID_SIZE -1);
        // start head is (1, 0)
        let head = snake.body.front().unwrap().clone();

        assert_eq!(head, head_should_be);
    }

    #[test]
    fn food_coordinates_in_bounds() {
        let food = Food::new_random(&LinkedList::from_iter(vec![]));

        assert!(food.x >= 0 && food.x < GRID_SIZE);
        assert!(food.y >= 0 && food.y < GRID_SIZE);
    }
}