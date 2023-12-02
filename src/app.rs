use ggez::event::EventHandler;
use ggez::graphics::{Color, DrawMode, DrawParam, Mesh};
use ggez::mint::{Point2, Vector2};
use ggez::winit::event::VirtualKeyCode;
use ggez::{graphics, Context};
use rand::Rng;

#[derive(Clone)]
struct Ball {
    point: Point2<f32>,
    radius: f32,
    velocity: Vector2<f32>,
    color: Color,
}

impl Ball {
    pub fn new(x: f32, y: f32, radius: f32, color: Color) -> Self {
        Self {
            point: Point2 { x, y },
            radius,
            velocity: Vector2 { x: 0.0, y: 0.0 },
            color,
        }
    }

    pub fn new_random(rng: &mut impl Rng, width: f32, height: f32) -> Self {
        let radius = rng.gen_range(10.0..50.0);
        let x = rng.gen_range(radius..width - radius);
        let y = rng.gen_range(radius..height - radius);
        Ball::new(x, y, radius, color!(?rng))
    }

    pub fn collides(&self, other: &Self) -> bool {
        let dx = self.point.x - other.point.x;
        let dy = self.point.y - other.point.y;
        let dist = (dx.powi(2) + dy.powi(2)).sqrt();
        dist <= self.radius + other.radius
    }

    pub fn collides_point(&self, point: Point2<f32>) -> bool {
        let dx = self.point.x - point.x;
        let dy = self.point.y - point.y;
        let dist = (dx.powi(2) + dy.powi(2)).sqrt();
        dist <= self.radius
    }

    pub fn move_from(&mut self, other: &Self) {
        let bounce = 0.05;
        let jump = 0.6;

        let dx = self.point.x - other.point.x;
        let dy = self.point.y - other.point.y;
        let angle = dy.atan2(dx);

        let dist = (dx.powi(2) + dy.powi(2)).sqrt();
        let force = self.radius + other.radius - dist;

        let x = angle.cos() * force;
        let y = angle.sin() * force;
        self.velocity.x += x * bounce * self.get_bounce_amount();
        self.velocity.y += y * bounce * self.get_bounce_amount();
        self.point.x += x * jump;
        self.point.y += y * jump;
    }

    pub fn get_bounce_amount(&self) -> f32 {
        let bounce_mass_falloff = 0.05;
        1.0 / (self.radius * bounce_mass_falloff).max(1.0)
    }
}

pub struct App {
    balls: Vec<Ball>,
    active_ball: Option<(usize, Point2<f32>)>,
}

impl App {
    pub fn new(ctx: &mut Context) -> Self {
        let (width, height) = ctx.gfx.drawable_size();

        let mut balls = Vec::new();
        let mut rng = rand::thread_rng();
        for _ in 0..10 {
            balls.push(Ball::new_random(&mut rng, width, height));
        }
        sort_balls_by_size(&mut balls);

        Self {
            balls,
            active_ball: None,
        }
    }

    pub fn reset(&mut self, ctx: &mut Context) {
        *self = Self::new(ctx);
    }

    fn move_active_ball(&mut self, x: f32, y: f32, vx: f32, vy: f32) {
        if let Some((i, offset)) = self.active_ball {
            let x = x - offset.x;
            let y = y - offset.y;
            let ball = &mut self.balls[i];
            ball.point = Point2 { x, y };
            ball.velocity = Vector2 { x: vx, y: vy };
        }
    }

    fn is_active_ball(&self, index: usize) -> bool {
        if let Some((i, _)) = self.active_ball {
            if i == index {
                return true;
            }
        }
        false
    }

    fn add_ball(&mut self, ball: Ball) {
        self.balls.push(ball);
        sort_balls_by_size(&mut self.balls);
    }
}

/// Sort list of balls largest to smallest
fn sort_balls_by_size(balls: &mut Vec<Ball>) {
    balls.sort_by(|a, b| b.radius.partial_cmp(&a.radius).unwrap());
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> Result<(), ggez::GameError> {
        let (width, height) = ctx.gfx.drawable_size();

        let bounce_amount = 0.5;

        for i in 0..self.balls.len() {
            if self.is_active_ball(i) {
                continue;
            }
            let ball = &mut self.balls[i];
            if ball.point.y + ball.radius < height {
                ball.velocity.y += 0.5
            }
        }

        for i in 0..self.balls.len() {
            if self.is_active_ball(i) {
                continue;
            }
            let ball = &mut self.balls[i];
            ball.point.x += ball.velocity.x;
            ball.point.y += ball.velocity.y;
        }

        for i in 0..self.balls.len() {
            if self.is_active_ball(i) {
                continue;
            }
            for j in 0..self.balls.len() {
                if i == j {
                    continue;
                }
                let ball = &self.balls[i];
                let other = self.balls[j].clone();

                if ball.collides(&other) {
                    let ball = &mut self.balls[i];
                    ball.move_from(&other);
                }
            }
        }

        for ball in &mut self.balls {
            if ball.point.x - ball.radius < 0.0 {
                ball.point.x = ball.radius;
                ball.velocity.x *= -bounce_amount * ball.get_bounce_amount();
            }
            if ball.point.x + ball.radius >= width {
                ball.point.x = width - ball.radius;
                ball.velocity.x *= -bounce_amount * ball.get_bounce_amount();
            }

            if ball.point.y + ball.radius >= height {
                ball.point.y = height - ball.radius;
                ball.velocity.y *= -bounce_amount * ball.get_bounce_amount();
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), ggez::GameError> {
        let mut canvas = graphics::Canvas::from_frame(ctx, color!(BLACK));

        for ball in &self.balls {
            let circle = Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                ball.point,
                ball.radius,
                0.1,
                ball.color,
            )?;
            canvas.draw(&circle, DrawParam::default());
        }

        canvas.finish(ctx)
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    ) -> Result<(), ggez::GameError> {
        self.move_active_ball(x, y, dx, dy);
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: ggez::event::MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        if self.active_ball.is_some() {
            return Ok(());
        }
        // Reverse to be sorted smallest to largest
        for i in (0..self.balls.len()).rev() {
            let ball = &self.balls[i];
            if ball.collides_point(Point2 { x, y }) {
                self.active_ball = Some((
                    i,
                    Point2 {
                        x: x - ball.point.x,
                        y: y - ball.point.y,
                    },
                ));
                self.move_active_ball(x, y, 0.0, 0.0);
                break;
            }
        }

        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: ggez::event::MouseButton,
        _x: f32,
        _y: f32,
    ) -> Result<(), ggez::GameError> {
        self.active_ball = None;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> Result<(), ggez::GameError> {
        let (width, height) = ctx.gfx.drawable_size();

        let Some(keycode) = input.keycode else {
            return Ok(());
        };

        match keycode {
            VirtualKeyCode::R => {
                self.reset(ctx);
            }
            VirtualKeyCode::Space => {
                self.add_ball(Ball::new_random(&mut rand::thread_rng(), width, height))
            }
            VirtualKeyCode::X => {
                if let Some((i, _)) = self.active_ball {
                    self.balls.remove(i);
                }
            }
            _ => (),
        }

        Ok(())
    }
}
