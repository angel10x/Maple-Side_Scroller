use macroquad::prelude::*;

const PLAYER_SIZE: Vec2 = Vec2::new(32.0, 48.0);
const PLAYER_SPEED: f32 = 200.0;
const JUMP_STRENGTH: f32 = 500.0;
const GRAVITY: f32 = 980.0;
const ENEMY_SIZE: Vec2 = Vec2::new(32.0, 32.0);
const ENEMY_SPEED: f32 = 50.0;

struct Player {
    pos: Vec2,
    velocity: Vec2,
    grounded: bool,
    facing_right: bool,
}

struct Platform {
    rect: Rect,
}

struct Enemy {
    pos: Vec2,
    velocity: Vec2,
    moving_right: bool,
    rect: Rect,
}

impl Player {
    fn new() -> Self {
        Self {
            pos: Vec2::new(100.0, 100.0),
            velocity: Vec2::ZERO,
            grounded: false,
            facing_right: true,
        }
    }

    fn update(&mut self, dt: f32, platforms: &[Platform]) {
        // Horizontal movement
        let mut acceleration = Vec2::ZERO;
        
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            acceleration.x -= PLAYER_SPEED;
            self.facing_right = false;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            acceleration.x += PLAYER_SPEED;
            self.facing_right = true;
        }

        // Jumping
        if (is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Space)) 
            && self.grounded {
            self.velocity.y = -JUMP_STRENGTH;
            self.grounded = false;
        }

        // Apply gravity
        self.velocity.y += GRAVITY * dt;
        
        // Apply horizontal movement
        self.velocity.x = acceleration.x;

        // Update position
        let new_pos = self.pos + self.velocity * dt;

        // Check collisions with platforms
        let player_rect = Rect::new(new_pos.x, new_pos.y, PLAYER_SIZE.x, PLAYER_SIZE.y);
        self.grounded = false;

        let mut final_pos = new_pos;

        for platform in platforms {
            if let Some(intersection) = player_rect.intersect(platform.rect) {
                // Resolve collision
                if self.velocity.y > 0.0 && intersection.y < intersection.x && intersection.y < intersection.w {
                    // Landing on top of platform
                    final_pos.y = platform.rect.y - PLAYER_SIZE.y;
                    self.velocity.y = 0.0;
                    self.grounded = true;
                } else if self.velocity.y < 0.0 && intersection.y > intersection.x {
                    // Hitting platform from below
                    final_pos.y = platform.rect.bottom();
                    self.velocity.y = 0.0;
                } else if self.velocity.x > 0.0 {
                    // Hitting from left
                    final_pos.x = platform.rect.x - PLAYER_SIZE.x;
                    self.velocity.x = 0.0;
                } else if self.velocity.x < 0.0 {
                    // Hitting from right
                    final_pos.x = platform.rect.right();
                    self.velocity.x = 0.0;
                }
            }
        }

        self.pos = final_pos;

        // Ground collision (prevent falling through bottom of screen)
        if self.pos.y > screen_height() - PLAYER_SIZE.y {
            self.pos.y = screen_height() - PLAYER_SIZE.y;
            self.velocity.y = 0.0;
            self.grounded = true;
        }

        // Apply friction when grounded
        if self.grounded {
            self.velocity.x *= 0.8;
        }
    }

    fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, PLAYER_SIZE.x, PLAYER_SIZE.y)
    }

    fn draw(&self) {
        // Simple player rectangle with direction indicator
        let color = if self.facing_right { BLUE } else { LIGHTGRAY };
        draw_rectangle(self.pos.x, self.pos.y, PLAYER_SIZE.x, PLAYER_SIZE.y, color);
        
        // Draw a simple face/eyes
        if self.facing_right {
            draw_circle(self.pos.x + 24.0, self.pos.y + 12.0, 3.0, BLACK);
            draw_circle(self.pos.x + 20.0, self.pos.y + 12.0, 3.0, BLACK);
        } else {
            draw_circle(self.pos.x + 8.0, self.pos.y + 12.0, 3.0, BLACK);
            draw_circle(self.pos.x + 12.0, self.pos.y + 12.0, 3.0, BLACK);
        }
    }
}

impl Enemy {
    fn new(x: f32, y: f32) -> Self {
        let pos = Vec2::new(x, y);
        Self {
            pos,
            velocity: Vec2::new(ENEMY_SPEED, 0.0),
            moving_right: true,
            rect: Rect::new(pos.x, pos.y, ENEMY_SIZE.x, ENEMY_SIZE.y),
        }
    }

    fn update(&mut self, dt: f32, platforms: &[Platform]) {
        // Simple patrol behavior - move back and forth
        if self.moving_right {
            self.velocity.x = ENEMY_SPEED;
        } else {
            self.velocity.x = -ENEMY_SPEED;
        }

        // Check if enemy should turn around at platform edges
        let check_pos = Vec2::new(
            self.pos.x + (if self.moving_right { ENEMY_SIZE.x + 5.0 } else { -5.0 }),
            self.pos.y + ENEMY_SIZE.y + 5.0,
        );
        
        let mut on_platform = false;
        for platform in platforms {
            if platform.rect.contains(check_pos) || check_pos.y >= screen_height() {
                on_platform = true;
                break;
            }
        }

        if !on_platform {
            self.moving_right = !self.moving_right;
        }

        // Apply gravity
        self.velocity.y += GRAVITY * dt;

        // Update position
        let new_pos = self.pos + self.velocity * dt;
        let enemy_rect = Rect::new(new_pos.x, new_pos.y, ENEMY_SIZE.x, ENEMY_SIZE.y);

        let mut final_pos = new_pos;
        
        // Check collisions with platforms
        for platform in platforms {
            if let Some(intersection) = enemy_rect.intersect(platform.rect) {
                if self.velocity.y > 0.0 && intersection.y < intersection.x {
                    // Landing on platform
                    final_pos.y = platform.rect.y - ENEMY_SIZE.y;
                    self.velocity.y = 0.0;
                }
            }
        }

        // Ground collision
        if final_pos.y > screen_height() - ENEMY_SIZE.y {
            final_pos.y = screen_height() - ENEMY_SIZE.y;
            self.velocity.y = 0.0;
        }

        self.pos = final_pos;
        self.rect = Rect::new(self.pos.x, self.pos.y, ENEMY_SIZE.x, ENEMY_SIZE.y);
    }

    fn draw(&self) {
        draw_rectangle(self.pos.x, self.pos.y, ENEMY_SIZE.x, ENEMY_SIZE.y, RED);
        // Draw simple eyes
        draw_circle(self.pos.x + 10.0, self.pos.y + 10.0, 3.0, YELLOW);
        draw_circle(self.pos.x + 22.0, self.pos.y + 10.0, 3.0, YELLOW);
    }
}

#[macroquad::main("MapleStory-style Side Scroller")]
async fn main() {
    let mut player = Player::new();
    
    // Create platforms
    let mut platforms = vec![
        Platform {
            rect: Rect::new(0.0, screen_height() - 40.0, screen_width(), 40.0),
        },
        Platform {
            rect: Rect::new(300.0, screen_height() - 200.0, 200.0, 20.0),
        },
        Platform {
            rect: Rect::new(600.0, screen_height() - 150.0, 150.0, 20.0),
        },
        Platform {
            rect: Rect::new(900.0, screen_height() - 300.0, 200.0, 20.0),
        },
        Platform {
            rect: Rect::new(1200.0, screen_height() - 250.0, 180.0, 20.0),
        },
    ];

    // Create enemies
    let mut enemies = vec![
        Enemy::new(400.0, screen_height() - 250.0),
        Enemy::new(700.0, screen_height() - 200.0),
        Enemy::new(1100.0, screen_height() - 350.0),
    ];

    let mut camera_x = 0.0;

    loop {
        clear_background(SKYBLUE);

        let dt = get_frame_time();

        // Update camera to follow player
        camera_x = (player.pos.x - screen_width() / 2.0).max(0.0);
        
        // Set camera transform
        set_camera(&Camera2D {
            target: vec2(camera_x + screen_width() / 2.0, screen_height() / 2.0),
            rotation: 0.0,
            zoom: vec2(1.0, 1.0),
            ..Default::default()
        });

        // Update player
        player.update(dt, &platforms);

        // Update enemies
        for enemy in &mut enemies {
            enemy.update(dt, &platforms);
        }

        // Draw platforms
        for platform in &platforms {
            draw_rectangle(platform.rect.x, platform.rect.y, platform.rect.w, platform.rect.h, GREEN);
            // Draw platform border
            draw_rectangle_lines(platform.rect.x, platform.rect.y, platform.rect.w, platform.rect.h, 2.0, DARKGREEN);
        }

        // Draw player
        player.draw();

        // Draw enemies
        for enemy in &enemies {
            enemy.draw();
        }

        // Draw UI (should not be affected by camera)
        set_camera(&Camera2D {
            target: vec2(screen_width() / 2.0, screen_height() / 2.0),
            rotation: 0.0,
            zoom: vec2(1.0, 1.0),
            ..Default::default()
        });

        // Draw instructions
        draw_text("WASD / Arrow Keys to move, Space to jump", 10.0, 30.0, 20.0, WHITE);

        next_frame().await;
    }
}