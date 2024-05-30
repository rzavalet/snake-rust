
use std::error::Error;
use std::time::Duration;
use rand::Rng;

extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render;
//use sdl2::render::Canvas;
use sdl2::video::Window;
//use sdl2::EventPump;
use sdl2::ttf;


const WIDTH       : u32 = 800;
const HEIGHT      : u32 = 600;
const SPACING     : u32 = 20;
const CELL_SPACE  : u32 = 20;

const NORMAL_SPEED: Duration = Duration::from_millis(200);
const FAST_SPEED:   Duration = Duration::from_millis(50);


/// Entry point. The path for the font file to use for rendering text in the game
/// must be passed as a string in `font_path`.
///
pub fn run(font_path: &str) -> Result<(), Box<dyn Error>> {
    let sdl_context = sdl2::init()?;
    let timer_subsystem = sdl_context.timer()?;
    let ttf_context = ttf::init().map_err(|e| e.to_string())?;
    let mut game = Game::new(&sdl_context, &timer_subsystem, &ttf_context, font_path);
    game.start();
    Ok(())
}


/// A particular `Game` can be in any of these states. We can think of they as different
/// screens in the game.
#[derive(Debug)]
enum GameState {
    STARTING,
    PLAYING,
    PAUSED,
    GAMEOVER,
}


/// In a given `GameState`, certain actions are possible. Those are denoted by these
/// `GameTransition`s.
#[derive(PartialEq, Debug)]
enum GameTransition {
    PLAY,
    PAUSE,
    LOSE,
    EXIT,
}


/// The area through which the snake can move is composed of cells. The area has `hcells` width
/// and `vcells` height.
struct GameArea {
    hcells      : u32,
    vcells      : u32,
    game_area   : Rect,
    /// SDL `Rect`angles conforming the game grid:
    grid        : Vec<Rect>,
}


/// A `Snake` can move in any of these directions. Well, that actually depends on the current
/// direction. E.g. if the `Snake` is moving `LEFT`, it cannot change its direction to `RIGHT`.
#[derive(PartialEq)]
enum Direction {
    LEFT,
    RIGHT,
    UP,
    DOWN,
}


/// The coordinates of a cell in the game display.
#[derive(Debug, Clone, Copy)]
struct Coordinate {
    x           : u32,
    y           : u32
}


/// A `Snake` is essentially a vector of cells in the grid, whose head is moving in certain
/// `direction`.
struct Snake {
    direction   :   Direction,
    body        :   Vec<Coordinate>,
}


/// We use the GameContext to stash anything related to the underlying SDL structures.
struct GameContext<'time> {
    _timer          : sdl2::timer::Timer<'time, 'time>,
    canvas          : sdl2::render::Canvas<Window>,
    event_pump      : sdl2::EventPump,
    current_state   : GameState,
}


impl<'time> GameContext<'time> {

    /// Constructor
    fn new(
        sdl_context: &'time sdl2::Sdl, timer_subsystem: &'time sdl2::TimerSubsystem,
    ) -> GameContext<'time>
    {
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("Simple Snake", WIDTH, HEIGHT)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();

        let event_pump    = sdl_context.event_pump().unwrap();
        let event_manager = sdl_context.event().unwrap();

        event_manager.register_custom_event::<TimerEvent>().unwrap();

        // `EventSender` objects can be moved to other threads and allow pushing
        // events to the queue from there:
        let event_sender = event_manager.event_sender();

        struct TimerEvent{} // No payload to carry.

        // Set a timer callback that pushes `TimerEvent` events.
        let _timer = timer_subsystem.add_timer(
            NORMAL_SPEED.as_millis().try_into().unwrap(),
            Box::new(move || -> u32 {
                // Queue next timer event. Note that there is no need to pause the timer,
                // since if an event of this same type is in the queue, the push operation is a no-op.
                event_sender.push_custom_event( TimerEvent{} ).unwrap();
                NORMAL_SPEED.as_millis().try_into().unwrap() // Return new interval.
            }
        ));

        GameContext {
            _timer,
            current_state : GameState::STARTING,
            canvas        : canvas,
            event_pump    : event_pump,
        }
    }
}


/// Generates the SDL `Rect`angle corresponding to the given game `coord`inate.
/// The `Rect` must fit inside `GameArea`; otherwise `None` is returned.
fn create_rect(display: &GameArea, coord: &Coordinate) -> Option<Rect> {

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
            let r = create_rect(&display, &Coordinate { x: hcell, y: vcell }).unwrap();
            display.grid.push(r);
        }
    }

    return display;
}


/// The actual state of the game.
struct Game<'ttf> {
    context     : GameContext<'ttf>,
    display     : GameArea,
    speed       : Duration,
    score       : u32,
    snake       : Snake,
    food        : Coordinate,

    score_rect  : Rect,
    font        : ttf::Font<'ttf, 'ttf>,
}


impl<'ttf> Game<'ttf> {
    // TODO: Implement the appropriate screens for `STARTING` and `PAUSED` states.

    /// Create a new `Game`. Once a game is created, a game can be `start`ed(). As part of creating
    /// the `Game`, its `GameContext` is also initialized. Such initialization consists of setting
    /// up everything related to SDL2.

    fn new(
        sdl_context: &'ttf sdl2::Sdl, timer_subsystem: &'ttf sdl2::TimerSubsystem,
        ttf_context: &'ttf ttf::Sdl2TtfContext, font_path: &str
    ) -> Game<'ttf>
    {
        let mut rng = rand::thread_rng();

        let mut font = ttf_context.load_font(font_path, 24)
            .expect("ERROR: Could not load font");

        font.set_style(ttf::FontStyle::BOLD);

        let display = create_grid();
        let snake = create_snake(&display);
        let ctxt = GameContext::new(sdl_context, timer_subsystem);

        let score_rect = Rect::new(SPACING as i32, 0, 100, SPACING);

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
            font    : font,
        };

        return game;
    }

    fn render_menu(&mut self, texture_creator: &render::TextureCreator<sdl2::video::WindowContext>, current_option: u32) {
        let options : Vec<&str> = vec![
            "  New Game",
            "> New Game",
            "  Exit",
            "> Exit",
        ];

        let new_game_message = options[1 - current_option as usize];
        let (fw1, fh1) = self.font.size_of(new_game_message).unwrap();

        let new_game_surface  = self.font
            .render(new_game_message)
            .solid(Color::RGB(0, 0, 0))
            .unwrap();
        let new_game_texture = texture_creator
            .create_texture_from_surface(&new_game_surface)
            .unwrap();
        let new_game_rect = Rect::new(WIDTH as i32/2 - fw1 as i32/2, HEIGHT as i32/2 - fh1 as i32/2, fw1, fh1);
        self.context.canvas.copy(&new_game_texture, None, Some(new_game_rect))
            .map_err(|e| e.to_string())
            .unwrap();

        let exit_message = options[2 + current_option as usize];
        let (fw2, fh2) = self.font.size_of(exit_message).unwrap();
        let exit_surface  = self.font
            .render(exit_message)
            .solid(Color::RGB(0, 0, 0))
            .unwrap();
        let exit_texture = texture_creator
            .create_texture_from_surface(&exit_surface)
            .unwrap();
        let exit_rect = Rect::new(WIDTH as i32/2 - fw1 as i32/2, 2 * SPACING as i32 + HEIGHT as i32/2 - fh2 as i32/2, fw2, fh2);
        self.context.canvas.copy(&exit_texture, None, Some(exit_rect))
            .map_err(|e| e.to_string())
            .unwrap();
    }


    /// Draws the menu, highlighting the option indexed by `current_option`
    fn draw_menu(&mut self, current_option: u32) {
        // FIXME: Should `texture_creator` be a field?
        let texture_creator = self.context.canvas.texture_creator();
        self.context.canvas.set_draw_color(Color::RGB(255, 255, 255));
        self.context.canvas.clear();
        self.context.canvas.set_draw_color(Color::RGB(255, 0, 0));
        self.context.canvas.draw_rect(self.display.game_area).unwrap();
        self.render_menu(&texture_creator, current_option);
        self.context.canvas.present();
    }


    /// Shows and manages the menu screen
    fn game_starting(&mut self) -> GameTransition {

        let mut current_option : u32 = 0;
        self.draw_menu(current_option);

        loop {
            let event = self.context.event_pump.wait_event();
            match event {
                Event::Quit{..} | Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    return GameTransition::EXIT;
                },

                Event::KeyDown { keycode: Some(Keycode::Up | Keycode::Down | Keycode::J | Keycode::K), ..} => {
                    // Update menu:
                    current_option = 1 - current_option;
                    self.draw_menu(current_option);
                },

                Event::KeyDown { keycode: Some(Keycode::Return), ..} => {
                    if current_option == 1 {
                        return GameTransition::EXIT;
                    }
                    else {
                        return GameTransition::PLAY;
                    }
                },
                _ => {}
            }
        }
    }


    /// Handles the paused loop. From here we can return to `PLAYING` or to `LOSE`
    ///
    fn paused_loop(&mut self) -> GameTransition {
        loop {
            let event = self.context.event_pump.wait_event();

            match event
            {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape | Keycode::Q), ..} => {
                    return GameTransition::LOSE;
                },

                Event::KeyDown { keycode: Some(Keycode::Space), ..} => {
                    return GameTransition::PLAY;
                },

                _ => {}
            }
        }
    }


    /// This is the loop for the `PLAYING` state. From this state we should be able to transition
    /// to either:
    ///     - PAUSED: If the user presses the space bar key.
    ///     - GAMEOVER: If the `Snake` collides with itself or with the walls.
    ///
    /// Otherwise, the game continues _ad infinitum`.
    /// 
    fn game_loop(&mut self) -> GameTransition {

        let mut draw_grid = false;

        loop {
            let event = self.context.event_pump.wait_event();

            match event
            {
                Event::User {..} => {
                    // The only user event we have is the timer, this means
                    // here we need to generate the current frame.
                    if ! self.draw_frame(draw_grid) {
                        return GameTransition::LOSE;
                    }
                },

                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape | Keycode::Q), ..} => {
                    return GameTransition::LOSE;
                },

                Event::KeyDown { keycode: Some(Keycode::Space), ..} => {
                    return GameTransition::PAUSE;
                },

                Event::KeyDown { keycode: Some(Keycode::Left | Keycode::H), ..} =>
                {
                    if self.snake.direction != Direction::RIGHT {
                        self.snake.direction = Direction::LEFT;
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::Right | Keycode::L), ..} =>
                {
                    if self.snake.direction != Direction::LEFT {
                        self.snake.direction = Direction::RIGHT;
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::Up | Keycode::K), ..} =>
                {
                    if self.snake.direction != Direction::DOWN {
                        self.snake.direction = Direction::UP;
                    }
                },
                
                Event::KeyDown { keycode: Some(Keycode::Down | Keycode::J), ..} =>
                {
                    if self.snake.direction != Direction::UP {
                        self.snake.direction = Direction::DOWN;
                    }
                },

                Event::KeyDown { keycode: Some(Keycode::Return), ..} => {
                    self.speed = FAST_SPEED;
                },

                Event::KeyUp { keycode: Some(Keycode::Return), ..} => {
                    self.speed = NORMAL_SPEED;
                },

                Event::KeyDown { keycode: Some(Keycode::G), ..} => {
                    // Toggle grid on and off
                    draw_grid = !draw_grid;
                },

                _ => {}
            }
        } // loop
    }


    /// Generates and draws the current frame.
    /// Return boolean indicating if the game can continue (or the user lost).
    ///
    fn draw_frame(&mut self, draw_grid:bool) -> bool{
        let mut rng = rand::thread_rng();

        let texture_creator = self.context.canvas.texture_creator();
        let score_surface : sdl2::surface::Surface;
        let texture : sdl2::render::Texture;

        self.context.canvas.set_draw_color(Color::RGB(255, 255, 255));
        self.context.canvas.clear();

        self.context.canvas.set_draw_color(Color::RGB(255, 0, 0));
        self.context.canvas.draw_rect(self.display.game_area).unwrap();

        if draw_grid {
            self.context.canvas.set_draw_color(Color::RGB(100, 100, 100));
            for r in &self.display.grid {
                self.context.canvas.draw_rect(*r).unwrap();
            }
        }

        let head = self.snake.body[0];
        let mut new_head = head;

        match self.snake.direction {
            Direction::LEFT  {..} => { 
                if new_head.x == 0 {
                    return false;
                }

                new_head.x -= 1; 
            },
            Direction::RIGHT {..} => { 
                if new_head.x == self.display.hcells - 1 {
                    return false;
                }

                new_head.x += 1; 
            },
            Direction::UP    {..} => { 
                if new_head.y == 0 {
                    return false;
                }

                new_head.y -= 1; 
            },
            Direction::DOWN  {..} => { 
                if new_head.y == self.display.vcells - 1 {
                    return false;
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

        for b in &self.snake.body[1..] {
            if new_head.x == b.x && new_head.y == b.y {
                return false;
            }
        }

        self.context.canvas.set_draw_color(Color::RGB(0,255,0));
        self.context.canvas.fill_rect(create_rect(&self.display, &self.snake.body[0])).unwrap();
        self.context.canvas.set_draw_color(Color::RGB(0,0,255));
        for b in &self.snake.body[1..] {
            self.context.canvas.fill_rect(create_rect(&self.display, b)).unwrap();
        }

        self.context.canvas.set_draw_color(Color::RGB(0,0,0));
        self.context.canvas.fill_rect(create_rect(&self.display, &self.food)).unwrap();

        let score_message = &format!("Score: {}", self.score);
        score_surface  = self.font
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
        return true;
    }

    /// This loop represents the `GAMEOVER` window that is shown when `GameState::PLAYING +
    /// GameTransition::LOSE` occurrs. 
    fn game_over_loop(&mut self) -> GameTransition {

        self.context.canvas.set_draw_color(Color::RGB(255, 255, 255));
        self.context.canvas.clear();

        self.context.canvas.set_draw_color(Color::RGB(255, 0, 0));
        self.context.canvas.draw_rect(self.display.game_area).unwrap();

        let texture_creator = self.context.canvas.texture_creator();
        let new_game_message = "You lost! Press any key to continue...";
        let (fw1, fh1) = self.font.size_of(new_game_message).unwrap();

        let new_game_surface  = self.font
            .render(new_game_message)
            .solid(Color::RGB(0, 0, 0))
            .unwrap();
        let new_game_texture = texture_creator
            .create_texture_from_surface(&new_game_surface)
            .unwrap();
        let new_game_rect = Rect::new(WIDTH as i32/2 - fw1 as i32/2, HEIGHT as i32/2 - fh1 as i32/2, fw1, fh1);
        self.context.canvas.copy(&new_game_texture, None, Some(new_game_rect))
            .map_err(|e| e.to_string())
            .unwrap();
        self.context.canvas.present();

        loop {
            let event = self.context.event_pump.wait_event();
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    return GameTransition::EXIT;
                },
                Event::KeyDown {..}  => {
                    return GameTransition::PLAY;
                },
                _ => {}
            }
        }
    }

    /// The `Game` is controlled by a `FSM`. This probably hasn't been fully thought through. Take
    /// it as a temporary skelleton for now. E.g. we currently don't have a `STARTING` window.
    /// TODO: Confirm that the FSM is complete.
    fn start(&mut self) {
        loop {
            let transition;// = GameTransition::EXIT;
            let mut handled = true; // Whether the transition was already processed by an inner state handle.
            // Unless explicitly set to false in a state handle, we assume the transition was already processed.

            match self.context.current_state {

                GameState::STARTING => {
                    transition = self.game_starting();
                    match transition
                    {
                        GameTransition::PLAY => {
                            self.context.current_state = GameState::PLAYING;
                        },
                        _ => { handled = false; }
                    }
                },

                GameState::PLAYING => {
                    transition = self.game_loop();
                    match transition
                    {
                        GameTransition::PAUSE => {
                            self.context.current_state = GameState::PAUSED;
                        },
                        GameTransition::LOSE => {
                            self.context.current_state = GameState::GAMEOVER;
                        },
                        _ => { handled = false; }
                    }
                },

                GameState::PAUSED => {
                    transition = self.paused_loop();
                    match transition
                    {
                        GameTransition::PLAY => {
                            self.context.current_state = GameState::PLAYING;
                        },
                        GameTransition::LOSE => {
                            self.context.current_state = GameState::GAMEOVER;
                        },
                        _ => { handled = false; }
                    }
                },

                GameState::GAMEOVER => {
                    transition = self.game_over_loop();
                    match transition
                    {
                        GameTransition::PLAY => {
                            self.context.current_state = GameState::STARTING;
                        },
                        _ => { handled = false; }
                    }
                }

            }

            if handled {
                continue;
            }
            else if transition == GameTransition::EXIT {
                // Global transition: finish game.
                return;
            }
            else {
                panic!("Invalid transition {:?} in state {:?}", transition, self.context.current_state);
            }
        }
    }
}
