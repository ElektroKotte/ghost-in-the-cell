use std::io;
use std::fmt::Write;

/**** Parsing help ****/

macro_rules! print_err {
    ($($arg:tt)*) => (
        {
            use std::io::Write;
            writeln!(&mut ::std::io::stderr(), $($arg)*).ok();
        }
    )
}

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

fn get_line() -> String {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    return buffer;
}

fn get_splitline() -> Vec<String> {
    get_line().split_whitespace().map(|tok| String::from(tok)).collect::<Vec<_>>()
}

/**** Game state helpers ****/

enum Command {
    Move { src: usize, dst: usize, bots: isize },
}

impl From<Command> for String {
    fn from(cmd: Command) -> Self {
        let mut out = String::new();
        match cmd {
            Command::Move { src, dst, bots } => {
                write!(&mut out, "MOVE {} {} {}", src, dst, bots);
            }
        }
        return out;
    }
}

struct Troop {
    owner: isize,
    bots: isize,
    turns: isize,
}

struct Factory {
    id: usize,
    owner: isize,
    bots: isize,
    production: isize,
    incoming: Vec<Troop>,
    distances: Vec<isize>,
}

impl Factory {
    fn new(id: usize, count: usize) -> Factory {
        Factory {
            id: id,
            owner: 0,
            bots: 0,
            production: 0,
            incoming: Vec::new(),
            distances: vec![0; count],
        }
    }
    
    fn clear(&mut self) {
        self.incoming.clear();
    }
}

struct GameState {
    factories: Vec<Factory>,
}

impl GameState {
    fn new(count: usize) -> GameState {
        let mut state = GameState {
            factories: Vec::new(),
        };
        for i in 0..count {
            state.factories.push(Factory::new(i, count));
        }
        return state;
    }
    
    fn clear(&mut self) {
        for ref mut f in self.factories.iter_mut() {
            f.clear();
        }
    }
}

fn main() {
    let factory_count = parse_input!(get_line(), usize);
    let link_count = parse_input!(get_line(), usize); 
    
    let mut state = GameState::new(factory_count);
    
    for i in 0..link_count {
        let inputs = get_splitline();
        let factory_1 = parse_input!(inputs[0], usize);
        let factory_2 = parse_input!(inputs[1], usize);
        let distance = parse_input!(inputs[2], isize);
        
        state.factories[factory_1].distances[factory_2] = distance;
        state.factories[factory_2].distances[factory_1] = distance;
    }

    /* Game loop */
    loop {
        /* Create clean slate for this turn */
        state.clear();
        let mut commands: Vec<Command> = Vec::new();
        
        /* Build game state */
        let entity_count = parse_input!(get_line(), usize);
        for i in 0..entity_count {
            let inputs = get_splitline();
            let id = parse_input!(inputs[0], usize);
            match inputs[1].as_str() {
                "FACTORY" => {
                    state.factories[id].owner = parse_input!(inputs[2], isize);
                    state.factories[id].bots = parse_input!(inputs[3], isize);
                    state.factories[id].production = parse_input!(inputs[4], isize);
                }
                "TROOP" => {
                    let target = parse_input!(inputs[4], usize);
                    state.factories[target].incoming.push( Troop {
                        owner: parse_input!(inputs[2], isize),
                        bots: parse_input!(inputs[5], isize),
                        turns: parse_input!(inputs[6], isize),
                    });
                }
                _ => /* BAD PROGRAM! Don't do that! */ ()
            }
        }

        /*
        /* Generate list of moves */
        for each factory
            if not owned by my
                - and has production, or is enemy
                - asses threat level,
                    - In how many turns can it be mine.
                    - at what cost
                    - Result is increased production, at cost of bots
            if owned by me
                - and has bots for enough to increase production.
                - add the move to increase output
                - Check threat level, does base need backup? Asses
                    as negative output at cost of 
        */    

        /* Find my base with most production */
        for i in 0..state.factories.len() {
            if state.factories[i].owner > 0 {
                let mut dests = {
                    let mut tmp = state.factories.iter().filter(|&f| {
                        let has_production = f.production > 0;
                        let isnt_me = f.owner <= 1;
                        let enemy = f.owner < 0;
                        (has_production && isnt_me) || enemy
                    }).collect::<Vec<&Factory>>();
                    tmp.sort_by_key(|&f| f.distances[i]);
                    tmp.iter().map(|&f| f.id).collect::<Vec<usize>>()
                };
                for &d in dests.iter() {
                    /* Dont attack same factory or myself */
                    if i == d { continue; }
                    if state.factories[d].owner > 0 { continue; }
                    
                    let bots = state.factories[i].bots / 2;
                    if state.factories[i].bots - bots >= state.factories[i].production {
                        state.factories[i].bots -= bots;
                        commands.push(Command::Move{src: i, dst: d, bots: bots});
                    }
                }
            }
        }
        
        if commands.len() > 0 {
            let output: String = commands.into_iter().map(String::from).collect::<Vec<String>>().join(";");
            println!("{}", output);
        } else {
            println!("WAIT");
        }
    }
}
