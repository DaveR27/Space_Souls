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

use agb::{include_aseprite,
    display::{object::{Graphics, Tag, ObjectController, Object, TagMap}, Priority},
    input::{self, Button, ButtonController}, fixnum::{Vector2D, FixedNum},
};

// type FixedNumberType = FixedNum<10>;

// pub struct Level {
//     background: &'static [u16],
//     foreground: &'static [u16],
//     dimensions: Vector2D<u32>,
//     collision: &'static [u32],

//     slimes: &'static [(i32, i32)],
//     snails: &'static [(i32, i32)],
//     enemy_stops: &'static [(i32, i32)],
//     start_pos: (i32, i32),
// }


// Import the sprites in to this constant. This holds the sprite 
// and palette data in a way that is manageable by agb.
const GRAPHICS: &Graphics = include_aseprite!("gfx/sprites.aseprite");
const TAG_MAP: &TagMap = GRAPHICS.tags();

// We define some easy ways of referencing the sprites
const SPACE_SHIP: &Tag = TAG_MAP.get("Space_Ship");
const ALIEN1: &Tag = TAG_MAP.get("Alien1");

pub struct Entity<'a> {
    sprite: Object<'a>,
    position: Vector2D<u16>,
    velocity: i32,
    collision_mask: Vector2D<u16>
}

impl <'a> Entity <'a> {
    pub fn new(object: &'a ObjectController, collision_mask: Vector2D<u16>) -> Self {
        let dummy_sprite = object.sprite(SPACE_SHIP.sprite(0));
        let mut sprite = object.object(dummy_sprite);
        sprite.set_priority(Priority::P1);
        Entity {
            sprite,
            collision_mask,
            position: Vector2D { x: (0), y: (0) },
            velocity: 1,
        }
    }
}

struct Player<'a> {
    space_ship: Entity<'a>,
    ammo: u8,
}

impl <'a> Player <'a> {
    fn new(controller: &'a ObjectController, start_position: Vector2D<u16>) -> Self {
        let mut space_ship = Entity::new(controller, (6_u16, 14_u16).into());
    
        space_ship.sprite
            .set_sprite(controller.sprite(SPACE_SHIP.sprite(0))); 

        space_ship.position = start_position;

        space_ship.sprite.set_x(space_ship.position.x).set_y(space_ship.position.y).show();


        Player {
            space_ship,
            ammo: 0,
        }
    }

    pub fn update_position(&mut self, new_x: u16) {
        self.space_ship.sprite.set_x(new_x);
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

    const BOTTOM_OF_SCREEN_OFFSET: u16 = 30;

    // Get the OAM manager
    let display_object = gba.display.object.get();

    // Create an object with the player_craft sprite
    let mut player_craft = Player::new(&display_object, 
        Vector2D { x: (50), y: (agb::display::HEIGHT as u16 - BOTTOM_OF_SCREEN_OFFSET) });

    let mut alien = display_object.object_sprite(ALIEN1.sprite(0));


    // Place craft on the screen
    // player_craft.set_x(50).set_y(agb::display::HEIGHT as u16 - BOTTOM_OF_SCREEN_OFFSET).show();

    alien.set_x(50).set_y(50).show();

    let mut player_craft_x = 50;
    let mut alien_x = 50;
    let mut x_velocity = 3;
    let mut ax_velo = 1;

loop {
    // This will calculate the new position and enforce the position
    // of the player_craft remains within the screen
    player_craft_x = (player_craft_x + x_velocity).clamp(0, agb::display::WIDTH - 16);

    // This will calculate the new position and enforce the position
    // of the alien remains within the screen
    alien_x = (alien_x + ax_velo).clamp(0, agb::display::WIDTH - 16);

    // We check if the player_craft reaches the edge of the screen and reverse it's direction
    if player_craft_x == 0 || player_craft_x == agb::display::WIDTH - 16 {
        x_velocity = -x_velocity;
    }

    // We check if the alien reaches the edge of the screen and reverse it's direction
    if alien_x == 0 || alien_x == agb::display::WIDTH - 16 {
        ax_velo = -ax_velo;
    }

    // Set the position of the player_craft and alien to match our new calculated position
    // For the player craft the Y doesn't matter
    player_craft.update_position(player_craft_x as u16);
    alien.set_x(alien_x as u16);

    // Wait for vblank, then commit the objects to the screen
    agb::display::busy_wait_for_vblank();
    display_object.commit();
    }
}
