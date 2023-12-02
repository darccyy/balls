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
        let bounce = 0.5;

        let dx = self.point.x - other.point.x;
        let dy = self.point.y - other.point.y;
        let angle = dy.atan2(dx);

        let dist = (dx.powi(2) + dy.powi(2)).sqrt();
        let force = (self.radius + other.radius - dist) * bounce;

        let x = angle.cos() * force;
        let y = angle.sin() * force;
        self.velocity.x += x;
        self.velocity.y += y;
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
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> Result<(), ggez::GameError> {
        let (width, height) = ctx.gfx.drawable_size();

        let bounce_amount = 0.5;
        let bounce_mass_falloff = 0.05;

        for i in 0..self.balls.len() {
            let ball = &mut self.balls[i];
            if ball.point.y + ball.radius < height {
                match self.active_ball {
                    Some((b, _)) if b == i => (),
                    _ => ball.velocity.y += 0.5,
                }
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
            ball.point.x += ball.velocity.x;
            ball.point.y += ball.velocity.y;

            if ball.point.x - ball.radius < 0.0 {
                ball.point.x = ball.radius;
                ball.velocity.x *= -bounce_amount;
                ball.velocity.x /= (ball.radius * bounce_mass_falloff).max(1.0);
            }
            if ball.point.x + ball.radius >= width {
                ball.point.x = width - ball.radius;
                ball.velocity.x *= -bounce_amount;
                ball.velocity.x /= (ball.radius * bounce_mass_falloff).max(1.0);
            }

            if ball.point.y + ball.radius >= height {
                ball.point.y = height - ball.radius;
                ball.velocity.y *= -bounce_amount;
                ball.velocity.y /= (ball.radius * bounce_mass_falloff).max(1.0);
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
        // Get smallest ball, which mouse is touching
        let mut colliding_balls = Vec::new();
        for (i, ball) in self.balls.iter().enumerate() {
            if ball.collides_point(Point2 { x, y }) {
                colliding_balls.push((i, ball));
            }
        }
        colliding_balls.sort_by(|(_, a), (_, b)| a.radius.partial_cmp(&b.radius).unwrap());
        if let Some((i, ball)) = colliding_balls.first() {
            self.active_ball = Some((
                *i,
                Point2 {
                    x: x - ball.point.x,
                    y: y - ball.point.y,
                },
            ));
            self.move_active_ball(x, y, 0.0, 0.0);
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
                self.balls
                    .push(Ball::new_random(&mut rand::thread_rng(), width, height))
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
