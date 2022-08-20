// Games made using `agb` are no_std which means you don't have access to the standard
// rust library. This is because the game boy advance doesn't really have an operating
// system, so most of the content of the standard library doesn't apply.
//
// Provided you haven't disabled it, agb does provide an allocator, so it is possible
// to use both the `core` and the `alloc` built in crates.
#![no_std]
// `agb` defines its own `main` function, so you must declare your game's main function
// using the #[agb::entry] proc macro. Failing to do so will cause failure in linking
// which won't be a particularly clear error message.
#![no_main]
// This is required to allow writing tests
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

use alloc::vec::Vec;
use agb::{include_aseprite,
          display::{object::{Graphics, Tag, ObjectController, Object, TagMap}, Priority},
          input::{self, Button, ButtonController}, fixnum::{Vector2D, FixedNum}, println,
};
use agb::display::tiled::{PartialUpdateStatus, RegularMap, TileIndex, VRamManager};
use agb::display::{HEIGHT, WIDTH};
use agb::hash_map::HashMap;
// use alloc::vec::Vec;

type FixedNumberType = FixedNum<10>;

pub struct Level {
    background: &'static [u16],
    foreground: &'static [u16],
    dimensions: Vector2D<u32>,
    collision: &'static [u32],

    enemy_stops: &'static [(i32, i32)],
    start_pos: (i32, i32),
}


// Import the sprites in to this constant. This holds the sprite 
// and palette data in a way that is manageable by agb.
const GRAPHICS: &Graphics = include_aseprite!("gfx/sprites.aseprite");
const TAG_MAP: &TagMap = GRAPHICS.tags();

// We define some easy ways of referencing the sprites
const SPACE_SHIP: &Tag = TAG_MAP.get("Space_Ship");
const ALIEN1: &Tag = TAG_MAP.get("Alien1");
// const MISSILE: &Tag = TAG_MAP.get("Missile");


pub struct Entity<'a> {
    sprite: Object<'a>,
    position: Vector2D<i32>,
    velocity: i32,
    collision_mask: Vector2D<u16>
}

impl <'a> Entity <'a> {
    pub fn new(object: &'a ObjectController, tag: &Tag, collision_mask: Vector2D<u16>) -> Self {
        let mut sprite = object.object(object.sprite(tag.sprite(0)));
        sprite.set_priority(Priority::P1);
        Entity {
            sprite,
            collision_mask,
            position: Vector2D { x: (0), y: (0) },
            velocity: 1,
        }
    }
}

pub struct Ammo {
    // ammo_entity: Entity,
    // entity_shooting: Entity <'a>,
}

// impl Ammo {
//     pub fn new(controller: &'a ObjectController, entity_shooting:  Entity <'a>) {
//         let mut ammo_entity = Entity::new(controller, MISSILE, (6_u16, 14_u16).into());

//         ammo_entity.position = entity_shooting.position;
//         ammo_entity.sprite.set_x(ammo_entity.position.x).set_y(ammo_entity.position.y).show();

//         // Ammo {
//         //     ammo_entity,
//         //     entity_shooting,
//         // }
//     }

//     // pub fn fire(controller: &'a ObjectController, entity_shooting:  Entity <'a>){
//     //     Ammo::new(controller, entity_shooting);
//     // }

//     // pub fn update_position(&mut self, new_position: Vector2D<u16>) {
//     //     self.ammo_entity.position = new_position;
//     //     self.ammo_entity.sprite.set_x(new_position.x).set_y(new_position.y);
//     // }
// }

struct Enemy<'a> {
    alien: Entity<'a>
}

impl <'a> Enemy <'a> {
    fn new(controller: &'a ObjectController, start_position: Vector2D<i32>) -> Self {
        let mut alien = Entity::new(controller, ALIEN1, (6_u16, 14_u16).into());
    
        alien.position = start_position;
        alien.sprite.set_x(alien.position.x as u16).set_y(alien.position.y as u16).show();

        Enemy {
            alien,
        }
    }

    pub fn update_position(&mut self, new_x: i32) {
        self.alien.position.x = new_x;
        self.alien.sprite.set_x(new_x as u16);
    }

    // Updates the alien position on the map
    pub fn update_frame(&mut self) {
        
        if self.alien.position.x == 0 || self.alien.position.x == agb::display::WIDTH - 16 {
            self.alien.velocity = -self.alien.velocity;
        }

        let new_x = self.alien.position.x + self.alien.velocity;
        self.update_position(new_x)
    }
}

struct Player<'a> {
    space_ship: Entity<'a>,
    // ammo: Vec<Ammo>,
}

impl <'a> Player <'a> {
    fn new(controller: &'a ObjectController) -> Self {
        const BOTTOM_OF_SCREEN_OFFSET: i32 = 30;
        let mut space_ship = Entity::new(controller, SPACE_SHIP, (6_u16, 14_u16).into());
    
        space_ship.position = Vector2D { x: (50), y: (agb::display::HEIGHT - BOTTOM_OF_SCREEN_OFFSET) };
        space_ship.sprite.set_x(space_ship.position.x as u16).set_y(space_ship.position.y as u16).show();

        Player {
            space_ship,
            // ammo: []
        }
    }

    pub fn update_position(&mut self, new_x: i32) {
        self.space_ship.position.x = new_x;
        self.space_ship.sprite.set_x(new_x as u16);
    }


    // Updates ther starships position on the map
    pub fn update_frame(
        &mut self,
        input: &ButtonController,
    ) {
        if input.is_pressed(Button::LEFT) {
            let new_x = if self.space_ship.position.x == 1 { self.space_ship.position.x } else { self.space_ship.position.x - 1};
            self.update_position(new_x);
        }
        if input.is_pressed(Button::RIGHT) {
            let new_x = if self.space_ship.position.x == agb::display::WIDTH - 16 { self.space_ship.position.x } else { self.space_ship.position.x + 1};
            self.update_position(new_x);
        }
        if input.is_pressed(Button::A) {
            // Ammo::new(controller, self);
        }
        // TODO: Update the ammo's frame
    }
}


struct PlayingLevel<'a> {
    timer: i32,
    // background: Map<'a>,
    input: ButtonController,
    player: Player<'a>,

    // enemies: [Enemy<'a>; 16],
}

impl<'a> PlayingLevel<'a> {
    fn open_level(
        level: &'a Level,
        object_control: &'a ObjectController,
        input: ButtonController,
    ) -> Self {
        // let mut e: [Enemy<'a>; 16] = Default::default();
        // let mut enemy_count = 0;
        // for &slime in level.slimes {
        //     e[enemy_count] = enemies::Enemy::new_slime(object_control, slime.into());
        //     enemy_count += 1;
        // }
        //
        // for &snail in level.snails {
        //     e[enemy_count] = enemies::Enemy::new_snail(object_control, snail.into());
        //     enemy_count += 1;
        // }

        // let background_position = (
        //     (start_pos.x - WIDTH / 2)
        //         .clamp(0.into(), ((level.dimensions.x * 8) as i32 - WIDTH).into()),
        //     (start_pos.y - HEIGHT / 2)
        //         .clamp(0.into(), ((level.dimensions.y * 8) as i32 - HEIGHT).into()),
        // )
        //     .into();

        PlayingLevel {
            timer: 0,
            player: Player::new(object_control),
            input,
            // enemies: e,
        }
    }
}


// The main function must take 1 arguments and never return. The agb::entry decorator
// ensures that everything is in order. `agb` will call this after setting up the stack
// and interrupt handlers correctly. It will also handle creating the `Gba` struct for you.
#[agb::entry]
fn gba_entry(mut gba: agb::Gba) -> ! {
    main(gba)
}

fn main(mut gba: agb::Gba) -> ! {
    let (tiled, mut vram) = gba.display.video.tiled0();

    // Get the OAM manager
    let display_object = gba.display.object.get();

    // Player input
    let mut button_object = agb::input::ButtonController::new();

    // Create an object with the player_craft sprite
    let mut player_craft = Player::new(&display_object);

    // Adds alien to the map
    let mut alien = Enemy::new(&display_object, 
        Vector2D { x: (50), y: (40) });


loop {
    // This will calculate the new position and enforce the position
    // of the player_craft remains within the screen
    button_object.update();
    player_craft.update_frame(&button_object);
    alien.update_frame();

    // Wait for vblank, then commit the objects to the screen
    agb::display::busy_wait_for_vblank();
    display_object.commit();
    }
}
