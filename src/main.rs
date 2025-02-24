#!([​warn​(​clippy​::all, ​clippy​::pedantic)])

use bracket_lib::prelude::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0; //updates the game physics in every 75ms
const DRAGON_START_X: i32 = 5;
const DRAGON_START_Y: i32 = 25;
const CENTER_HEADLINE: i32 = 5;
const CENTER_OPTION_1: i32 = 8;
const CENTER_OPTION_2: i32 = 9;
const CENTER_OPTION_3: i32 = 10;
const GAP_Y_MIN: i32 = 10;
const GAP_Y_MAX: i32 = 40;

enum GameMode {
    Menu,
    Playing,
    End,
}

struct Obstacle {
    x: i32,
    gap_y: i32, //specifying at which position of the wall a gap should occur for the dragon to go through
    size: i32,  //size of the gap
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(GAP_Y_MIN, GAP_Y_MAX), //randomly selecting the position of gap
            size: i32::max(2, 20 - score), //increasing difficyulty by reducing the size as dragon advances
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let distance_x = self.x - player_x;
        let half_size = self.size / 2;

        for y in 0..self.gap_y - half_size {
            //drawing the top half oof  the wall
            ctx.set(distance_x, y, RED, BLACK, to_cp437('|'));
        }

        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            //drawing the bottom half oof  the wall
            ctx.set(distance_x, y, RED, BLACK, to_cp437('|'));
        }
    }

    fn hit_obstacle(&mut self, dragon: &Dragon) -> bool {
        let x_matching = self.x == dragon.x;
        let half_size = self.size / 2;
        let y_top_matching = dragon.y < self.gap_y - half_size;
        let y_bottom_matching = dragon.y > self.gap_y + half_size;
        x_matching && (y_top_matching || y_bottom_matching)
    }
}

struct Dragon {
    x: i32,
    y: i32,
    velocity: f32,
}

impl Dragon {
    fn new(x: i32, y: i32) -> Self {
        Dragon {
            x,
            y,
            velocity: 0.0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(0, self.y, YELLOW, BLACK, to_cp437('@'));
    }

    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2; //gradually applying gravity against the upward momentum
                                  //remember, the greater the velocity the dragon goes downwards
        }
        self.y += self.velocity as i32; //remember 0 is the top of the screen, so if the velocity is -ve the dragon flies upward
        self.x += 1; //incrementing the x pos of dragon
        if self.y < 0 {
            //keeping the dragon within the box
            self.y = 0;
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.0; // increasing the upward momentum of dragon
                              //self.velocity -= 2.0; can do like this as well but it will take two flaps for the dragon to gain upward momentum if its already falling
    }
}

struct State {
    mode: GameMode,
    dragon: Dragon,
    frame_time: f32, //time between each frame refresh
    obstacle: Obstacle,
    score: i32,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::End => self.dead(ctx),
        }
    }
}

impl State {
    fn new() -> Self {
        State {
            mode: GameMode::Menu,
            dragon: Dragon::new(DRAGON_START_X, DRAGON_START_Y),
            frame_time: 0.0,
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            score: 0,
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.dragon.gravity_and_move();
        }

        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.dragon.flap();
        }

        self.dragon.render(ctx);
        ctx.print(0, 0, "press SPACE to flap");
        ctx.print(0, 1, &format!("Your Score {}", self.score));

        self.obstacle.render(ctx, self.dragon.x);

        if self.dragon.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(self.dragon.x + SCREEN_WIDTH, self.score)
        }

        if self.dragon.y > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.dragon) {
            self.mode = GameMode::End;
        }
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls(); // clears the screen
        ctx.print_centered(CENTER_HEADLINE, "Welcome to Flappy Dragon"); //printing the options in menu
        ctx.print_centered(CENTER_OPTION_1, "(P) Play Game");
        ctx.print_centered(CENTER_OPTION_2, "(Q) Quit Game");
        // ctx.print_centered(10, "(R) Res Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(CENTER_HEADLINE, "You are Dead!");
        ctx.print_centered(CENTER_OPTION_1, &format!("Your Score {}", self.score));
        ctx.print_centered(CENTER_OPTION_2, "(P) Play Again");
        ctx.print_centered(CENTER_OPTION_3, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn restart(&mut self) {
        self.dragon = Dragon::new(DRAGON_START_X, DRAGON_START_Y);
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0);
        self.frame_time = 0.0;
        self.score = 0;
        self.mode = GameMode::Playing;
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple(SCREEN_WIDTH, SCREEN_HEIGHT)?
        .with_title("Flappy Dragon")
        .build()?;

    main_loop(context, State::new())
}
