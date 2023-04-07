#![cfg_attr(not(test), no_std)]

use bare_metal_modulo::{ModNumC, MNum, ModNumIterator};
use pluggable_interrupt_os::vga_buffer::{BUFFER_WIDTH, BUFFER_HEIGHT, plot, ColorCode, Color, is_drawable, plot_num, plot_str, clear, clear_screen};
use pc_keyboard::{DecodedKey, KeyCode};
use num::traits::SaturatingAdd;
use core::default::Default;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand::RngCore;

const NEW_ENEMY_FREQ: isize = 100;

const WALLS: &str = 
"################################################################################
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
################################################################################";


pub struct Game{
    game_over: bool,
    player: Entity,
    tick_count: isize,
    rng: SmallRng,
    enemies: [Option<Entity>;BUFFER_WIDTH], 
    walls: Map,
    score: usize,
    new_enemy_spawn_index: usize,
    last_enemy_killed_index: usize
}

impl Game{
    pub fn new() -> Self {
        let player = Entity::default();
        Self {
            game_over: true,
            player: player,
            tick_count: 0, 
            rng: SmallRng::seed_from_u64(3),
            enemies: [None; BUFFER_WIDTH], 
            walls: Map::new(WALLS, Color::LightGreen),
            score: 0,
            new_enemy_spawn_index: 0,
            last_enemy_killed_index: 0
        }
    }

    pub fn key(&mut self, key: DecodedKey) {
        let mut future = self.player;
        match key {
            DecodedKey::RawKey(key) => {
                match key {
                    KeyCode::ArrowUp => {
                        future.up();
                    },
                    KeyCode::ArrowDown => {
                        future.down();
                    },
                    KeyCode::ArrowLeft => {
                        future.left();
                    },
                    KeyCode::ArrowRight => {
                        future.right();
                    },
                    _ => {}
                }
                
            }
            DecodedKey::Unicode(key) => {
                match key {
                    'w' => {
                        future.up();
                    },
                    's' => {
                        future.down();
                    },
                    'a' => {
                        future.left();
                    },
                    'd' => {
                        future.right();
                    },
                    'r' => {
                        if self.game_over{
                            self.game_over = false;
                            let temp = Entity::default();
                            self.player = temp;
                            self.enemies = [None; BUFFER_WIDTH];
                            self.score = 0;
                            self.rng = SmallRng::seed_from_u64(self.tick_count as u64);
                            clear_screen();
                        }
                    },
                    '`' => {
                        self.game_over = true;
                        clear_screen();
                    },
                    _ => {}
                }
            }
        }
        let enemy_check = self.is_colliding_enemy(future);
        self.enemy_move();
        if !future.is_colliding(&self.walls) && enemy_check.is_none() {
            plot(' ', self.player.x, self.player.y, ColorCode::new(Color::Black, Color::Black));
            self.player = future;
            if self.player.current_health < self.player.max_health as isize{
                self.player.current_health += 1;
            }
        }
        else if enemy_check.is_some(){
            self.attack_enemy(enemy_check.unwrap());
        }
    }

    fn get_enemy_index(&mut self, mut enemy: Entity) -> isize{
        for i in 0..self.enemies.len() {
            if self.enemies[i].is_some(){
                if enemy.equal_to(self.enemies[i].unwrap()){
                    return i as isize;
                }
            }
        }
        return -1;
    }  

    fn attack_enemy(&mut self, mut enemy: Entity){
        enemy.current_health -= self.player.damage as isize - enemy.defense as isize;
        self.player.current_health -= enemy.damage as isize - self.player.defense as isize;
        
        if enemy.current_health <= 0 {
            self.score += 100;
            let enemy_index = self.get_enemy_index(enemy);
            if enemy_index != -1{
                self.enemies[enemy_index as usize] = None;
                self.last_enemy_killed_index = enemy_index as usize;
            }
        }

        if self.player.current_health <= 0{
            self.game_over = true;
        }
    }

    fn is_colliding_enemy(&mut self, future: Entity) -> Option<Entity>{
        for enemy in self.enemies{
            if enemy.is_some(){
                let unwrap = enemy.unwrap();
                if (self.player.x + self.player.attack_range == unwrap.x || self.player.x - self.player.attack_range == unwrap.x) && self.player.y == unwrap.y{
                    let mut current_distance = 0;
                    let mut new_distance = 0;
                    if self.player.x > unwrap.x{
                        current_distance = self.player.x - unwrap.x;
                        new_distance = future.x - unwrap.x;
                    }
                    else{
                        current_distance = unwrap.x - self.player.x;
                        new_distance = unwrap.x - future.x;
                    }
                    if new_distance < current_distance{
                        return Some(unwrap);
                    }
                }
                if (self.player.y + self.player.attack_range == unwrap.y || self.player.y - self.player.attack_range == unwrap.y) && self.player.x == unwrap.x{
                    let mut current_distance = 0;
                    let mut new_distance = 0;
                    if self.player.y > unwrap.y{
                        current_distance = self.player.y - unwrap.y;
                        new_distance = future.y - unwrap.y;
                    }
                    else{
                        current_distance = unwrap.y - self.player.y;
                        new_distance = unwrap.y - future.y;
                    }
                    if new_distance < current_distance{
                        return Some(unwrap);
                    }
                }
            }
        }
        return None;
    }

    fn enemy_move(&mut self){
        for enemy in self.enemies{
            if enemy.is_some(){
                let past = enemy.unwrap();
                let mut future = enemy.unwrap();
                let index = self.get_enemy_index(past);
                let random_dir = 1 + self.rng.next_u32() as usize % 4;
                match random_dir{
                    1 => {future.up();},
                    2 => {future.down();},
                    3 => {future.right();},
                    4 => {future.left();},
                    _ => {}
                }
                if !future.is_colliding(&self.walls){
                    plot(' ', past.x, past.y, ColorCode::new(Color::Black, Color::Black));
                    self.enemies[index as usize] = Some(future);
                }
            }
        }
    }

    fn draw(&mut self){
        self.walls.draw('#');
        plot(self.player.char, self.player.x, self.player.y, ColorCode::new(Color::Green, Color::Black));
        plot_str("HP: ", 10, 1, ColorCode::new(Color::LightRed, Color::Black));
        plot_num(self.player.current_health as isize, 14, 1, ColorCode::new(Color::LightRed, Color::Black));
        plot_str("Attack: ", 35, 1, ColorCode::new(Color::LightRed, Color::Black));
        plot_num(self.player.damage as isize, 43, 1, ColorCode::new(Color::LightRed, Color::Black));
        plot_str("Defense: ", 60, 1, ColorCode::new(Color::LightRed, Color::Black));
        plot_num(self.player.defense as isize, 69, 1, ColorCode::new(Color::LightRed, Color::Black));
        for enemy in self.enemies{
            if enemy.is_some(){
                let temp = enemy.unwrap();
                plot(temp.char, temp.x, temp.y, ColorCode::new(temp.color, Color::Black));
            }
        }
    }

    pub fn tick(&mut self){
        if !self.game_over{
            self.tick_count += 1;
            self.draw();
            if self.tick_count % NEW_ENEMY_FREQ == 0{
                if self.enemies[self.last_enemy_killed_index].is_some(){
                    self.enemies[self.new_enemy_spawn_index] = Some(self.player.generate_stats(1 + self.rng.next_u32() as usize % 4, 1 + self.rng.next_u32() as usize % BUFFER_WIDTH - 1, 1 + self.rng.next_u32() as usize % BUFFER_HEIGHT - 1));
                    self.new_enemy_spawn_index += 1;
                }
                else{
                    self.enemies[self.last_enemy_killed_index] = Some(self.player.generate_stats(1 + self.rng.next_u32() as usize % 4, 1 + self.rng.next_u32() as usize % BUFFER_WIDTH -1, 1 + self.rng.next_u32() as usize % BUFFER_HEIGHT - 1));
                }
            }
        }
        else{
            self.tick_count += 1;
            plot_str("Definitely Not Rogue", BUFFER_WIDTH/2-12 , 5, ColorCode::new(Color::LightBlue, Color::Black));
            plot_str("Press r to begin and ` to quit", BUFFER_WIDTH/2-17, 7, ColorCode::new(Color::LightBlue, Color::Black));
            plot_str("Previous Game Score: ", BUFFER_WIDTH/2-12, BUFFER_HEIGHT/2+5, ColorCode::new(Color::LightGreen, Color::Black));
            plot_num(self.score as isize, BUFFER_WIDTH/2-3, BUFFER_HEIGHT/2+6, ColorCode::new(Color::LightGreen, Color::Black));
        }
    }
}

#[derive(Copy,Clone)]
pub struct Entity{
    char: char,
    x: usize,
    y: usize,
    max_health: usize,
    current_health: isize,
    damage: usize,
    defense: usize,
    attack_range: usize,
    attack_width: usize,
    weapon: Option<Item>,
    armor: Option<Item>,
    color: Color
}

impl Default for Entity{
    fn default() -> Self{
        Self{
            char: 'A',
            x: BUFFER_WIDTH / 2,
            y: BUFFER_HEIGHT / 2,
            max_health: 10,
            current_health: 10,
            damage: 2,
            defense: 0,
            attack_range: 1,
            attack_width: 1,
            weapon: None,
            armor: None,
            color: Color::LightBlue
        }
    }
}

impl Entity {
    pub fn new(character: char, x: usize, y: usize, max_health: usize, damage: usize, defense: usize, attack_range: usize, attack_width: usize, weapon: Option<Item>, armor: Option<Item>, color: Color) -> Self {
        Self {
            char: character,
            x: x,
            y: y,
            max_health: max_health, 
            current_health: max_health as isize,
            damage: damage,
            defense: defense, 
            attack_range: attack_range,
            attack_width: attack_width,
            weapon: weapon,
            armor: armor,
            color: color
        }
    }

    fn draw(&self) {
        plot(self.char, self.x, self.y, ColorCode::new(self.color, Color::Black));
    }

    fn generate_stats(&self, enemy_type: usize, x: usize, y: usize) -> Entity{
        match enemy_type{
            //skeleton 
            1 => {
                let skeleton = Entity::new(
                    'S',
                    x,
                    y,
                    7,
                    2,
                    1,
                    2,
                    1,
                    None,
                    None,
                    Color::Red
                );
                return skeleton;
            },
            //werewolf
            2 => {
                let werewolf = Entity::new(
                    'W',
                    x,
                    y,
                    15,
                    4,
                    0,
                    1,
                    1,
                    None,
                    None,
                    Color::Red
                );
                return werewolf;
            },
            //archer
            3 => {
                let archer = Entity::new(
                    'B',
                    x,
                    y,
                    7,
                    3,
                    1,
                    3,
                    1,
                    None,
                    None,
                    Color::Red
                );
                return archer;
            },
            //porcupine
            4 => {
                let porcupine = Entity::new(
                    'P',
                    x,
                    y,
                    10,
                    1,
                    2,
                    1,
                    3,
                    None,
                    None,
                    Color::Red
                );
                return porcupine;
            },
            _ => {
                let mut temp = Entity::default();
                temp.color = Color::Red;
                temp.char = 'a';
                return temp;
            }
        }
    }

    fn equal_to(&mut self, entity: Entity) -> bool{
        if self.x != entity.x || self.y != entity.y{
            return false;
        }
        return true;
    }

    fn is_colliding(&self, walls: &Map) -> bool {
        walls.occupied(self.y, self.x)
    }

    fn down(&mut self) {
        self.y += 1;
    }

    fn up(&mut self) {
        self.y -= 1;
    }

    fn left(&mut self) {
        self.x -= 1;
    }

    fn right(&mut self) {
        self.x += 1;
    }
}

#[derive(Copy, Clone)]
pub struct Item{
    char: char,
    health_change: usize,
    damage_change: usize,
    defense_change: usize,
    attack_range_change: usize,
    attack_width_change: usize
}

#[derive(Copy, Clone)]
pub struct Map{
    color: Color,
    items: [[bool; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

impl Default for Map {
    fn default() -> Self {
        Self { color: Color::White, items: [[false; BUFFER_WIDTH]; BUFFER_HEIGHT]}
    }
}

impl Map {
    pub fn new(map: &str, color: Color) -> Self {
        let mut walls = [[false; BUFFER_WIDTH]; BUFFER_HEIGHT];
        for (row, chars) in map.split('\n').enumerate() {
            for (col, value) in chars.char_indices() {
                walls[row][col] = value == '#';
            }
        }
        Self {items: walls, color}
    }

    pub fn add_random_item(&mut self, rng: &mut SmallRng) {
        let col: usize = 1 + rng.next_u32() as usize % (BUFFER_WIDTH - 1);
        let row: usize = 1 + rng.next_u32() as usize % (BUFFER_HEIGHT - 1);
        self.add(row, col);
    }

    pub fn change_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn draw(&self, character: char) {
        for row in 0..self.items.len() {
            for col in 0..self.items[row].len() {
                if self.occupied(row, col) {
                    plot(character, col, row, ColorCode::new(self.color, Color::Black));
                }
            }
        }
    }

    pub fn occupied(&self, row: usize, col: usize) -> bool {
        self.items[row][col]
    }

    pub fn add(&mut self, row: usize, col: usize) {
        self.items[row][col] = true;
    }

    pub fn remove(&mut self, row: usize, col: usize) {
        self.items[row][col] = false;
    }

    fn char_at(&self, row: usize, col: usize) -> char {
        if self.items[row][col] {
            '#'
        } else {
            ' '
        }
    }
}
