extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;
use sdl2::ttf;

use std::time::Duration;

use rand::Rng;

const WIDTH       : u32 = 800;
const HEIGHT      : u32 = 600;
const SPACING     : u32 = 20;
const CELL_SPACE  : u32 = 20;

const NORMAL_SPEED: Duration = Duration::from_millis(300);
const FAST_SPEED:   Duration = Duration::from_millis(50);

const FONT_FILE : &str = "./res/Roboto-Regular.ttf";
//const FONT_FILE = "/usr/share/fonts/truetype/roboto/unhinted/RobotoTTF/Roboto-Regular.ttf";

#[derive(Debug)]
enum GameState {
    /// A particular `Game` can be in any of these states. We can thing of they as different
    /// screens in the game.
    STARTING,
    PLAYING,
    PAUSED,
    GAMEOVER,
}


#[derive(Debug)]
enum GameTransition {
    /// In a given `GameState`, certain actions are possible. Those are denoted by these
    /// `GameTransition`s.
    PLAY,
    PAUSE,
    LOOSE,
    EXIT,
}

struct GameArea {
    /// The area through which the snake can move is composed of cells. The area has `hcells` width
    /// and `vcells` height.
    hcells      : u32,
    vcells      : u32,
    game_area   : Rect,
    grid        : Vec<Rect>,
}

enum Direction {
    /// A `Snake` can move in any of these directions. Well, that actually depends on the current
    /// direction. E.g. if the `Snake` is moving `LEFT`, it cannot change its direction to `RIGHT`.
    LEFT,
    RIGHT,
    UP,
    DOWN,
}

#[derive(Debug, Clone, Copy)]
struct Coordinate {
    x           : u32,
    y           : u32
}

struct Snake {
    /// A `Snake` is essentially a vector of cells in the grid, whose head is moving in certain
    /// `direction`.
    direction   :   Direction,
    body        :   Vec<Coordinate>,
}

struct GameContext {
    /// We use the GameContext to stash anything related to the underlying SDL structures.
    current_state   : GameState,
    canvas          : Canvas<Window>,
    event_pump      : EventPump,
}

impl GameContext {
    fn new() -> GameContext {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let mut window = video_subsystem.window("Simple Snake", WIDTH, HEIGHT)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();

        let event_pump = sdl_context.event_pump().unwrap();

       GameContext {
           current_state : GameState::STARTING,
           canvas        : canvas,
           event_pump    : event_pump,
       }
    }

}

/// Generates a `Rec`tangle whose units are in pixels.
fn rec_generator(display: &GameArea, coord: &Coordinate) -> Option<Rect> {

    if coord.x > display.hcells {
        return None;
    }

    if coord.y > display.vcells {
        return None;
    }

    let r = Rect::new(((1 + coord.x) * CELL_SPACE) as i32, 
                      ((1 + coord.y) * CELL_SPACE) as i32, 
                      CELL_SPACE, 
                      CELL_SPACE);

    return Some(r);
}

/// Create a `Snake` with a certain number of cells as its body. Let's always initialize its
/// direction to `RIGHT` for now.
fn create_snake(display: &GameArea) -> Snake {
    let mut snake = Snake {
        direction: Direction::RIGHT,
        body     : Vec::new(),
    };

    snake.body.push(
        Coordinate{
            x: display.hcells/2,
            y: display.vcells/2});

    snake.body.push(
        Coordinate{
            x: display.hcells/2 - 1, 
            y: display.vcells/2});

    snake.body.push(
        Coordinate{
            x: display.hcells/2 - 2, 
            y: display.vcells/2});

    snake.body.push(
        Coordinate{
            x: display.hcells/2 - 3, 
            y: display.vcells/2});

    snake.body.push(
        Coordinate{
            x: display.hcells/2 - 4, 
            y: display.vcells/2});

    return snake;
}

/// As explained earlier, GameArea is a grid of cells. Here we create such cells as rectangles.
fn create_grid() -> GameArea {
    let mut display = GameArea {
        vcells   : (HEIGHT - 2 * SPACING) / SPACING,
        hcells   : (WIDTH  - 2 * SPACING) / SPACING,
        game_area: Rect::new(SPACING as i32, 
                             SPACING as i32, 
                             WIDTH  - 2 * SPACING, 
                             HEIGHT - 2 * SPACING),
        grid: Vec::new(),
    };

    for vcell in 0..display.vcells {
        for hcell in 0..display.hcells {
            let r = rec_generator(&display, &Coordinate { x: hcell, y: vcell }).unwrap();
            display.grid.push(r);
        }
    }

    return display;
}


struct Game {
    context     : GameContext,
    // TODO: We need to figure out how to set the `lifetime`s of multiple structures. This one is
    // only an example. There are many more.
    display     : GameArea,
    speed       : Duration,
    score       : u32,
    snake       : Snake,
    food        : Coordinate,

    score_rect  : Rect,
    game_over_rect: Rect,
}

impl Game {
    // TODO: Implement the appropriate screens for `STARTING` and `PAUSED` states.

    /// Create a new `Game`. Once a game is created, a game can be `start`ed(). As part of creating
    /// the `Game`, its `GameContext` is also initialized. Such initialization consists of setting
    /// up everything related to SDL2.
    fn new() -> Game {
        let mut rng = rand::thread_rng();
        let display = create_grid();
        let snake = create_snake(&display);
        let ctxt = GameContext::new();

        let score_rect = Rect::new(SPACING as i32, 0, 100, SPACING);
        let game_over_rect = Rect::new((WIDTH/2) as i32, (HEIGHT/2) as i32, 100, SPACING);

        let food = Coordinate {
            x : rng.gen_range(0..display.hcells), 
            y : rng.gen_range(0..display.vcells),
        };

        let game = Game {
            context : ctxt,
            display : display,
            speed   : NORMAL_SPEED,
            score   : 0,
            snake   : snake,
            food    : food,
            score_rect : score_rect,
            game_over_rect : game_over_rect,
        };

        return game;
    }

    fn render_menu(&mut self, font: &ttf::Font, texture_creator: &render::TextureCreator<sdl2::video::WindowContext>, current_option: &mut u32) {
        let options : Vec<&str> = vec![
            "  New Game",
            "> New Game",
            "  Exit",
            "> Exit",
        ];

        let new_game_message = options[1 - *current_option as usize];
        let (fw1, fh1) = font.size_of(new_game_message).unwrap();

        let mut new_game_surface  = font
            .render(new_game_message)
            .solid(Color::RGB(0, 0, 0))
            .unwrap();
        let mut new_game_texture = texture_creator
            .create_texture_from_surface(&new_game_surface)
            .unwrap();
        let new_game_rect = Rect::new(WIDTH as i32/2 - fw1 as i32/2, HEIGHT as i32/2 - fh1 as i32/2, fw1, fh1);
        self.context.canvas.copy(&new_game_texture, None, Some(new_game_rect))
            .map_err(|e| e.to_string())
            .unwrap();

        let exit_message = options[2 + *current_option as usize];
        let (fw2, fh2) = font.size_of(exit_message).unwrap();
        let mut exit_surface  = font
            .render(exit_message)
            .solid(Color::RGB(0, 0, 0))
            .unwrap();
        let mut exit_texture = texture_creator
            .create_texture_from_surface(&exit_surface)
            .unwrap();
        let exit_rect = Rect::new(WIDTH as i32/2 - fw1 as i32/2, 2 * SPACING as i32 + HEIGHT as i32/2 - fh2 as i32/2, fw2, fh2);
        self.context.canvas.copy(&exit_texture, None, Some(exit_rect))
            .map_err(|e| e.to_string())
            .unwrap();

    }

    fn game_starting(&mut self) -> GameTransition {

        let mut rng = rand::thread_rng();

        let mut result : GameTransition = GameTransition::EXIT;

        let texture_creator = self.context.canvas.texture_creator();

        // TODO: `font` should be another field in `GameContext`. No need to load the font in every
        // `GameState`. Fixing this requires knowledge on `Lifetimes`, though.
        let ttf_context = ttf::init().map_err(|e| e.to_string()).unwrap();
        let mut font = ttf_context.load_font(FONT_FILE, 24).expect("ERROR: Could not load font");
        font.set_style(ttf::FontStyle::BOLD);

        let mut current_option : u32 = 0;

        'running: loop {
            self.context.canvas.set_draw_color(Color::RGB(255, 255, 255));
            self.context.canvas.clear();

            self.context.canvas.set_draw_color(Color::RGB(255, 0, 0));
            self.context.canvas.draw_rect(self.display.game_area).unwrap();

            for event in self.context.event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                        break 'running
                    },
                    Event::KeyDown { keycode: Some(Keycode::Up), ..} |
                    Event::KeyDown { keycode: Some(Keycode::Down), ..} => {
                        current_option = 1 - current_option;
                    },
                    Event::KeyDown { keycode: Some(Keycode::Return), ..} => {
                        if current_option == 1 {
                            result = GameTransition::EXIT;
                            break 'running;
                        }
                        else {
                            result = GameTransition::PLAY;
                            break 'running;
                        }
                    },

                    _ => {}
                }
            }

            self.render_menu(&font, &texture_creator, &mut current_option);
            self.context.canvas.present();
            ::std::thread::sleep(self.speed);

        }

        return result;
    }

    /// This is the loop for the `PLAYING` state. From this state we should be able to transition
    /// to either:
    ///     - PAUSED: If the user presses _some_ key.
    ///     - GAMEOVER: If the `Snake` collides with itself or with the walls.
    ///
    /// Otherwise, the game continues _ad infinitum`.
    /// 
    fn game_loop(&mut self) -> GameTransition {

        let mut result = GameTransition::LOOSE;
        let mut rng = rand::thread_rng();

        let texture_creator = self.context.canvas.texture_creator();
        let mut score_surface : sdl2::surface::Surface;
        let mut texture : sdl2::render::Texture;

        // TODO: `font` should be another field in `GameContext`. No need to load the font in every
        // `GameState`. Fixing this requires knowledge on `Lifetimes`, though.
        let ttf_context = ttf::init().map_err(|e| e.to_string()).unwrap();
        let mut font = ttf_context.load_font(FONT_FILE, 24).expect("ERROR: Could not load font");
        font.set_style(ttf::FontStyle::NORMAL);


        'running: loop {
            self.context.canvas.set_draw_color(Color::RGB(255, 255, 255));
            self.context.canvas.clear();

            self.context.canvas.set_draw_color(Color::RGB(255, 0, 0));
            self.context.canvas.draw_rect(self.display.game_area).unwrap();

            self.context.canvas.set_draw_color(Color::RGB(0, 0, 0));
            for r in &self.display.grid {
                self.context.canvas.draw_rect(*r).unwrap();
            }

            for event in self.context.event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                        result = GameTransition::LOOSE;
                        break 'running
                    },
                    Event::KeyDown { keycode: Some(Keycode::Left), ..} => {
                        match self.snake.direction {
                            Direction::RIGHT {..}   => {},
                            _                       => {self.snake.direction = Direction::LEFT;}
                        }
                    },
                    Event::KeyDown { keycode: Some(Keycode::Right), ..} => {
                        match self.snake.direction {
                            Direction::LEFT {..}    => {},
                            _                       => {self.snake.direction = Direction::RIGHT;}
                        }
                    },
                    Event::KeyDown { keycode: Some(Keycode::Up), ..} => {
                        match self.snake.direction {
                            Direction::DOWN {..}    => {},
                            _                       => {self.snake.direction = Direction::UP;}
                        }
                    },
                    Event::KeyDown { keycode: Some(Keycode::Down), ..} => {
                        match self.snake.direction {
                            Direction::UP {..}      => {},
                            _                       => {self.snake.direction = Direction::DOWN;}
                        }
                    },
                    Event::KeyDown { keycode: Some(Keycode::Return), ..} => {
                        self.speed = FAST_SPEED;
                    },
                    Event::KeyUp { keycode: Some(Keycode::Return), ..} => {
                        self.speed = NORMAL_SPEED;
                    },

                    _ => {}
                }
            }

            let head = self.snake.body[0];
            let mut new_head = head;

            match self.snake.direction {
                Direction::LEFT  {..} => { 
                    if new_head.x == 0 {
                        result = GameTransition::LOOSE;
                        break 'running;
                    }

                    new_head.x -= 1; 
                },
                Direction::RIGHT {..} => { 
                    if new_head.x == self.display.hcells - 1 {
                        result = GameTransition::LOOSE;
                        break 'running;
                    }

                    new_head.x += 1; 
                },
                Direction::UP    {..} => { 
                    if new_head.y == 0 {
                        result = GameTransition::LOOSE;
                        break 'running;
                    }

                    new_head.y -= 1; 
                },
                Direction::DOWN  {..} => { 
                    if new_head.y == self.display.vcells - 1 {
                        result = GameTransition::LOOSE;
                        break 'running;
                    }

                    new_head.y += 1; 
                },
            }

            if new_head.x == self.food.x && new_head.y == self.food.y {
                self.food.x = rng.gen_range(0..self.display.hcells);
                self.food.y = rng.gen_range(0..self.display.vcells);
                self.score += 1;
                //println!("New score: {0}", self.score);
            }
            else {
                self.snake.body.pop().unwrap();
            }

            self.snake.body.insert(0,new_head);

            //for b in &self.snake.body[1..] {
            //    println!("Body {0} {1}", b.x, b.y);
            //}
            //println!("New Head {0} {1}", new_head.x, new_head.y);

            for b in &self.snake.body[1..] {
                if new_head.x == b.x && new_head.y == b.y {
                    result = GameTransition::LOOSE;
                    break 'running;
                }
            }


            self.context.canvas.set_draw_color(Color::RGB(0,255,0));
            self.context.canvas.fill_rect(rec_generator(&self.display, &self.snake.body[0])).unwrap();
            self.context.canvas.set_draw_color(Color::RGB(0,0,255));
            for b in &self.snake.body[1..] {
                self.context.canvas.fill_rect(rec_generator(&self.display, b)).unwrap();
            }

            self.context.canvas.set_draw_color(Color::RGB(0,0,0));
            self.context.canvas.fill_rect(rec_generator(&self.display, &self.food)).unwrap();

            let score_message = &format!("Score: {}", self.score);
            score_surface  = font
                .render(score_message)
                .solid(Color::RGB(0, 0, 0))
                .unwrap();
            texture = texture_creator
                .create_texture_from_surface(&score_surface)
                .unwrap();
            self.context.canvas.copy(&texture, None, Some(self.score_rect))
                .map_err(|e| e.to_string())
                .unwrap();

            self.context.canvas.present();
            ::std::thread::sleep(self.speed);
        }

        return result;
    }

    /// This loop represents the `GAMEOVER` window that is shown when `GameState::PLAYING +
    /// GameTransition::LOOSE` occurrs. 
    ///
    /// The idea is that it should show the option to `EXIT` or `PLAY` again.
    fn game_over(&mut self) -> GameTransition {

        //let transition = GameTransition::PLAY;

        let texture_creator = self.context.canvas.texture_creator();
        let mut score_surface : sdl2::surface::Surface;
        let mut texture : sdl2::render::Texture;

        // TODO: `font` should be another field in `GameContext`. No need to load the font in every
        // `GameState`. Fixing this requires knowledge on `Lifetimes`, though.
        let ttf_context = ttf::init().map_err(|e| e.to_string()).unwrap();
        let mut font = ttf_context.load_font(FONT_FILE, 24).expect("ERROR: Could not load font");
        font.set_style(ttf::FontStyle::NORMAL);


        'gameover: loop {
            self.context.canvas.set_draw_color(Color::RGB(255, 255, 255));
            self.context.canvas.clear();

            self.context.canvas.set_draw_color(Color::RGB(255, 0, 0));
            self.context.canvas.draw_rect(self.display.game_area).unwrap();

            self.context.canvas.set_draw_color(Color::RGB(0, 0, 0));
            for r in &self.display.grid {
                self.context.canvas.draw_rect(*r).unwrap();
            }

            for event in self.context.event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                        //transition = GameTransition::EXIT;
                        break 'gameover
                    },
                    Event::KeyDown { keycode: Some(Keycode::Return), ..} => {
                        //transition = GameTransition::EXIT;
                        break 'gameover
                    },

                    _ => {}
                }
            }

            score_surface  = font
                .render("Game Over")
                .solid(Color::RGB(255, 0, 0))
                .unwrap();
            texture = texture_creator
                .create_texture_from_surface(&score_surface)
                .unwrap();
            self.context.canvas.copy(&texture, None, Some(self.game_over_rect))
                .map_err(|e| e.to_string())
                .unwrap();

            self.context.canvas.present();
            ::std::thread::sleep(self.speed);
        }

        //return transition;
        return GameTransition::EXIT;
    }


    fn game_over_loop(&mut self) -> GameTransition {
        let mut rng = rand::thread_rng();

        let mut result : GameTransition = GameTransition::EXIT;

        let texture_creator = self.context.canvas.texture_creator();

        // TODO: `font` should be another field in `GameContext`. No need to load the font in every
        // `GameState`. Fixing this requires knowledge on `Lifetimes`, though.
        let ttf_context = ttf::init().map_err(|e| e.to_string()).unwrap();
        let mut font = ttf_context.load_font(FONT_FILE, 24).expect("ERROR: Could not load font");
        font.set_style(ttf::FontStyle::BOLD);

        let mut current_option : u32 = 0;

        'running: loop {
            self.context.canvas.set_draw_color(Color::RGB(255, 255, 255));
            self.context.canvas.clear();

            self.context.canvas.set_draw_color(Color::RGB(255, 0, 0));
            self.context.canvas.draw_rect(self.display.game_area).unwrap();

            for event in self.context.event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                        result = GameTransition::EXIT;
                        break 'running
                    },
                    Event::KeyDown {..}  => {
                        result = GameTransition::PLAY;
                        break 'running;
                    },

                    _ => {}
                }
            }


            let new_game_message = "You lost! Press any key to continue...";
            let (fw1, fh1) = font.size_of(new_game_message).unwrap();

            let mut new_game_surface  = font
                .render(new_game_message)
                .solid(Color::RGB(0, 0, 0))
                .unwrap();
            let mut new_game_texture = texture_creator
                .create_texture_from_surface(&new_game_surface)
                .unwrap();
            let new_game_rect = Rect::new(WIDTH as i32/2 - fw1 as i32/2, HEIGHT as i32/2 - fh1 as i32/2, fw1, fh1);
            self.context.canvas.copy(&new_game_texture, None, Some(new_game_rect))
                .map_err(|e| e.to_string())
                .unwrap();

            self.context.canvas.present();
            ::std::thread::sleep(self.speed);

        }

        return result;
    }

    /// The `Game` is controlled by a `FSM`. This probably hasn't been fully thought through. Take
    /// it as a temporary skelleton for now. E.g. we currently don't have a `STARTING` window.
    /// TODO: Confirm that the FSM is complete.
    fn start(&mut self) {
        'game: loop {
            match self.context.current_state {
                GameState::STARTING {..} => {
                    let transition = self.game_starting();
                    match transition {
                        GameTransition::PLAY {..} => {
                            self.context.current_state = GameState::PLAYING;
                        },

                        GameTransition::EXIT {..} => {
                            break 'game
                        },

                        _ => {
                            panic!("Invalid transition {:?} in state {:?}", transition, self.context.current_state);
                        }
                    }
                },

                GameState::PLAYING {..} => {
                    let transition = self.game_loop();
                    match transition {
                        GameTransition::PAUSE {..} => {
                            self.context.current_state = GameState::PAUSED;
                        },

                        GameTransition::LOOSE {..} => {
                            self.context.current_state = GameState::GAMEOVER;
                        },

                        _ => {
                            panic!("Invalid transition {:?} in state {:?}", transition, self.context.current_state);
                        }
                    }

                },

                GameState::PAUSED {..} => {
                    let transition = GameTransition::PLAY;
                    match transition {
                        GameTransition::PLAY {..} => {
                            self.context.current_state = GameState::PLAYING;
                        },

                        GameTransition::EXIT {..} => {
                            break 'game
                        },

                        _ => {
                            panic!("Invalid transition {:?} in state {:?}", transition, self.context.current_state);
                        }
                    }
                },

                GameState::GAMEOVER {..} => {
                    let transition = self.game_over_loop();
                    match transition {
                        GameTransition::PLAY {..} => {
                            self.context.current_state = GameState::STARTING;
                        },

                        GameTransition::EXIT {..} => {
                            break 'game
                        },

                        _ => {
                            panic!("Invalid transition {:?} in state {:?}", transition, self.context.current_state);
                        }
                    }
                },
            } // match current_state
        } // end loop
    }
}



fn main() {

    let mut game = Game::new();
    game.start();
}
