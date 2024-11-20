use tcod::colors::*;
use tcod::console::*;
use std::cmp;

// actual size of the window
const SCREEN_WIDTH: i32 = 160;
const SCREEN_HEIGHT: i32 = 90;

// size of the map
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const COLOR_DARK_WALL : Color = Color {r: 0, g:0, b: 100};
const COLOR_DARK_GROUND : Color = Color {
    r: 50,
    g: 50,
    b: 150,
};

// A tile of the map and its properties
#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
        }
    }
}

type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
}

const LIMIT_FPS: i32 = 20;

struct Tcod {
    root: Root,
    con: Offscreen,
}

// handle keys
fn handle_keys(tcod: &mut Tcod, player: &mut Object, game: &Game) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    // czekam na kliknięcie klawisza
    let key = tcod.root.wait_for_keypress(true);
    // w zależności od tego, który klawisz został wciśnięty
    // wykonuje operacje
    match key {
        Key {code: Up, ..} => player.move_by(0, 1, game),
        Key {code: Down, ..} => player.move_by(0, -1, game),
        Key {code: Right, ..} => player.move_by(1, 0, game),
        Key {code: Left, ..} => player.move_by(-1, 0, game),
        Key {code: Enter, ctrl: true, ..} => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        },
        Key {code: Escape, ..} => {
            return true
        }

        _ => {}
    }

    false
}

#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object {x, y, char, color}
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
        if !game.map[(self.x + dx) as usize][(self.y - dy) as usize].blocked {

            self.x += dx;
            self.y -= dy;
        }
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32, 
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }
}

fn create_room(room: Rect, map: &mut Map) {
    for x in (room.x1+1)..room.x2 {
        for y in (room.y1+1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    // horizontal tunnel. min(), max() are used in case x1 > x2
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    // horizontal tunnel. min(), max() are used in case x1 > x2
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn make_map() -> Map {
    // fill map with "unblocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    
    let room1 = Rect::new(20, 15, 10, 15);
    let room2 = Rect::new(50, 15, 10, 15);
    create_h_tunnel(25, 55, 23, &mut map);
    create_room(room1, &mut map);
    create_room(room2, &mut map);
    map
}

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object]){
    // draw all objects in the list
    for object in objects{
        object.draw(&mut tcod.con);
    }

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = game.map[x as usize][y as usize].block_sight;
            if wall {
                tcod.con
                .set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                tcod.con
                .set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
            }
        }
    }
    blit(
        &tcod.con,
        (0, 0),
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );
}


fn main() {

    let player = Object::new(25, 23, '@', WHITE);
    let npc = Object::new(MAP_WIDTH / 2 - 10, MAP_HEIGHT / 2 + 10, '@', YELLOW);

    // oryginalny player i npc, przestają istnieć w momencie przeniesienia do tablicy
    let mut objects = [player, npc];
    
    tcod::system::set_fps(LIMIT_FPS);

    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Roguelike rust game")
        .init();

    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);

    let mut tcod = Tcod { root, con };

    let game = Game {
        map: make_map(),
    };

    // tworzymy okno gry
    while !tcod.root.window_closed() {
        tcod.con.clear();
        render_all(&mut tcod, &game, &objects);
        tcod.root.flush();
        let player = &mut objects[0];
        let exit = handle_keys(&mut tcod, player, &game);
        if exit {
            break;
        }
        
    }



}
